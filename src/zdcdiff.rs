use anyhow::Result;
use piwis_val::{Measurement, ValueEnum, VehicleAnalysisLog, Section};
use piwis_zdc::{HumanTranslations, InstrLst, Instr};
use indexmap::{IndexMap, IndexSet};
use serde::de::value;
use std::collections::{HashMap, HashSet};

#[derive(clap::Args, Debug)]
pub struct ZdcDiffArgs {
    dir: String,
    zip: Option<String>,
    #[clap(long)]
    #[arg(default_value_t = true)]
    include_zdc_file: bool,
    #[clap(long)]
    #[arg(default_value_t = true)]
    include_diag_addr: bool,
    #[clap(long)]
    #[arg(default_value_t = true)]
    include_phase: bool,
    #[clap(long)]
    #[arg(default_value_t = true)]
    include_srv_name: bool,
    #[clap(long)]
    #[arg(default_value_t = true)]
    compare_phases: bool,
}

#[derive(Debug, Default)]
pub struct ZdcDiffConfig {
    pub(crate) include_zdc_file: bool,
    pub(crate) include_diag_addr: bool,
    pub(crate) include_phase: bool,
    pub(crate) include_srv_name: bool,
}

impl ZdcDiffConfig {
    pub fn new(include_zdc_file: bool, include_diag_addr: bool, include_phase: bool, include_srv_name: bool, compare_phases: bool) -> ZdcDiffConfig {
        ZdcDiffConfig {
            include_zdc_file,
            include_diag_addr,
            include_phase,
            include_srv_name,
        }
    }
}


fn get_target_value_by_name(diag_addr: &str, target_name: &str) -> Option<String> {
    // Example: Predefined mapping of diag_addr and target_name to target values
    let predefined_values: HashMap<(&str, &str), String> = HashMap::from([
        (("003C", "CodingValue.Bitfield_Param_RearAxleSteer"), "with_Rear_Axle_Steerin".to_string()),
        (("0069", "CodingValue.Param_StatuTermi30OutpuPin9"), "activated_trailer_mode".to_string()),
        (("0069", "CodingValue.Bitfield2_Param_DimmuFLED"), "activated_trailer_mode".to_string()),
        (("0036", "CodingValue.Bitfield_Param_VehicWithoPasseSSG"), "passenger_SSG_installed".to_string()),
        (("0036", "CodingValue.Bitfield2_Param_LocatLordoButto"), "not_active".to_string()),
        (("0604", "TABROW_GaragDoorOpeneSuppoDoorName.Bitfield_Param_GaragDoorOpeneSuppoDoorName"), "Bubba".to_string()),
        ]);

        // Attempt to find the target value in the map
        predefined_values.get(&(diag_addr, target_name)).cloned()
}

fn group_instr_by_diag_addr(instr_lsts: &[InstrLst]) -> IndexMap<String, Vec<&InstrLst>> {
    let mut instr_lst_groups: IndexMap<String, Vec<&InstrLst>> = IndexMap::new();
    for instr_lst in instr_lsts {
        instr_lst_groups
            .entry(instr_lst.diag_addr.value.clone())
            .or_insert_with(Vec::new)
            .push(instr_lst);
    }
    instr_lst_groups
}

fn dedup_instr_files(instr_lsts: &[&InstrLst]) -> Vec<String> {
    instr_lsts.iter()
        .map(|z| z.zdc_file.clone())
        .collect::<IndexSet<_>>() // Deduplicate using IndexSet
        .into_iter()
        .collect()
}

fn build_instr_maps(instr_lsts: &[&InstrLst]) -> IndexMap<(String, String), Vec<(String, String)>> {
    let mut parameter_map: IndexMap<(String, String), Vec<(String, String)>> = IndexMap::new();
    
    for instr_lst in instr_lsts {
        for instr in &instr_lst.lst {
            if let Some(service) = InstrLst::extract_service(instr) {
                if let Some(translations) = &service.transl {
                    
                    let srv_name = translations
                    .srv_name
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string());
                    
                    if let Some(params) = &translations.params {

                        for param in params {
                            parameter_map
                                .entry((srv_name.clone(), param.name.clone()))
                                .or_insert_with(Vec::new)
                                .push((service.phase.clone(), param.value.clone()));
                        } 
                        
                    } else if let Some(data_sets) = &translations.data_sets {
                        
                        for data_set in data_sets {                            
                            parameter_map
                                .entry((format!("DATASET.{}",data_set.rd_id.clone()),data_set.srv_name.clone()))
                                .or_insert_with(Vec::new)                                                         
                                .push((service.phase.clone(), data_set.value.clone()));
                        }
                    }                                 
                } 
            }
        }
    }
    parameter_map
}

fn process_maps(
    diag_addr: &str,
    parameter_map: IndexMap<(String, String), Vec<(String, String)>>,
    val: Option<&VehicleAnalysisLog>)
 {
    for ((srv_name, name), value_list) in &parameter_map {
                
        let mut unique_values = HashSet::new();
        let mut all_phases = vec![];
        let mut all_values = vec![];
    
        for (phase, value) in value_list {          
            
            all_phases.push(phase.clone());
            all_values.push(value.clone());
            unique_values.insert(value.clone());
           
        }
        
        let unique_values: Vec<_> = unique_values.into_iter().collect();
  
        let target_name = format!("{}.{}", srv_name, name);

        let target_value;

        match val {
            Some(val) => {
                target_value = get_target_value_by_name(diag_addr, &target_name);
            }
            None => {
                target_value = None;
            }
        }

        match target_value {
            None => match unique_values.len() {
                1 => {                    
                    if all_values.len() > 1 {
                        println!(
                            "[ZDC-MATCH] [{}] >> {}.{}: {}",
                            all_phases.join(" -> "),
                            srv_name,
                            name,
                            all_values.join(" -> "),                          
                        );
                    } else {
                        println!(
                            "[ZDC-SINGLE] [{}] >> {}.{}: {}",
                            all_phases.join(" -> "),
                            srv_name,
                            name,
                            all_values.join(" -> "),        
                        );
                    }
                }
                _ => {
                    println!(
                        "[ZDC-DIFF] [{}] >> {}.{}: {}",
                        all_phases.join(" -> "),
                        srv_name,
                        name,
                        all_values.join(" -> "),
                    );
                }
            },
            Some(ref target_val) => {

                if unique_values.len() == 1 && unique_values[0] == *target_val {
                    println!(
                        "[ZDC-VAL-MATCH] [{} => VAL] >> {}.{}: {} => {}",
                        all_phases.join(" -> "),
                        srv_name,
                        name,
                        all_values.join(" -> "),                       
                        target_val
                    );
                } else {
                    println!(
                        "[ZDC-VAL-DIFF] [{} => VAL] >> {}.{}: {} => {}",
                        all_phases.join(" -> "),
                        srv_name,
                        name,
                        all_values.join(" -> "),           
                        target_val
                    );
                }
            }
        }
    }
}

fn build_val_maps(val: Option<&VehicleAnalysisLog>) -> IndexMap<(String, String), IndexMap<String, Vec<String>>> {
    
    let mut val_map: IndexMap<(String, String), IndexMap<String, Vec<String>>> = IndexMap::new();

    for section in val.unwrap().result.sections.iter() {
        let mut p0 = vec![];
        p0.push(section.get_title().clone());
   //     print_val_params(&mut p0, &section.get_measurements());
        p0.pop();
    }
    val_map
}

fn compare_instr_val(
    instr_lsts: &[InstrLst],
    val: Option<&VehicleAnalysisLog>)
    {
    
    let instr_lst_groups = group_instr_by_diag_addr(instr_lsts);

    for (diag_addr, instr_lsts) in instr_lst_groups {
 
        // Build zdc parameter 
        let instr_lst_param_map = build_instr_maps(&instr_lsts);
        
        if !instr_lst_param_map.is_empty() {
               // Deduplicate zdc_files
               let unique_files = dedup_instr_files(&instr_lsts);

               // Print diag_addr and deduplicated zdc_files
               println!(
                   "Diagnosis Address: {} >> ZDC: {}",
                   diag_addr,
                   unique_files.join(",")
               );
            
            process_maps(&diag_addr, instr_lst_param_map, val);
        }
    }
}


pub fn zdcdiff(args: &ZdcDiffArgs) -> Result<()> {
    
    let zdc_dump_config = &mut ZdcDiffConfig::new(args.include_zdc_file,
        args.include_diag_addr,
        args.include_phase,
        args.include_srv_name,
        args.compare_phases);

    let instr_lsts = &InstrLst::from_dir(&args.dir)?;
  
    if let Some(zip) = &args.zip {

        let val = &VehicleAnalysisLog::from_zip(&zip)?;
        
        // Build val parameter 
      //  let val_param_map = build_val_maps(Some(val));
      
        let val_param_map =  group_val_by_diag_addr(Some(val)); /* -> IndexMap<String, Section>*/ 


        // Handle the case where the zip argument is provided
        compare_instr_val(
            &instr_lsts, 
            Some(&val));

    } else {
        
        // Handle the case where the zip argument is not provided
        compare_instr_val(
            &instr_lsts,
            None);
    }

  
    Ok(())
}

fn print_val_value(p0: &mut Vec<String>, values: &Option<&Vec<ValueEnum>>) {

    match (&values)
    {
        Some(values) => { 
            for value in *values {
                p0.push(value.get_label().clone());
                p0.push(value.get_label().clone());
                println!("{}: {}",p0.join(" -> "), value.get_value().unwrap_or(&"<undefined>".to_string()));
                p0.pop();
                p0.pop();
            }
        }
        None=> {
            return;
        }
    }

}
    
fn print_val_params(p0: &mut Vec<String>, measurements: &Vec<Measurement>) {
        
    for measurement in measurements {
        p0.push(measurement.get_title().clone());

        match &measurement.get_submeasurements() {
            Some(nested_measurements) =>
            {
                print_val_params(p0, nested_measurements);
            }
            _ => (),
        }
        print_val_value(p0, &measurement.get_values());
        p0.pop();
    }
}    

fn group_val_by_diag_addr(val: Option<&VehicleAnalysisLog>) -> HashMap<String, Vec<String>> {
    let mut val_groups: HashMap<String, Vec<String>> = HashMap::new();

    if let Some(vehicle_log) = val {
        for section in &vehicle_log.result.sections {
            let m = &section.get_measurement_by_title(&"Identification".to_string()).unwrap();

            let value_txt = get_section_diagr_addr(section, &m);
            
            println!("Section: {} -> {} -> {}", section.get_title(), m.get_title(), value_txt);

            val_groups.entry(value_txt.clone())
                .or_insert_with(Vec::new)
                .push((*section.get_title()).clone());
        }
    }

    val_groups
}

fn get_section_diagr_addr(section: &Section, m: &Measurement) -> String {

    let labels = [
        "TABROW_SubsyIdent.Param_SubsyIdentID",
        "Gateway_Identification.Gateway_Identification_Diagnose_ID_VW",
        "TABROW_BusmaIdent.Param_ECUID",
        "System_Identification.System_Identification_Subsystem_ID",
        "System_Identification.System_Identification_Param_SubsyIdentID",
    ];

    let value_txt = labels.iter()
        .filter_map(|label| m.get_value_by_label(&label.to_string()))
        .map(|value| value.get_value().unwrap_or(&"<undefined>".to_string()).to_string())
        .next()
        .unwrap_or_else(|| "<undefined>".to_string());

   
    
    value_txt
}