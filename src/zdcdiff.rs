use anyhow::Result;
use piwis_val::{Measurement, ValueEnum, VehicleAnalysisLog};
use piwis_zdc::{HumanTranslations, InstrLst, Instr};
use std::collections::{HashMap, HashSet};
use itertools::Itertools;

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

fn group_instr_lst_by_diag_addr(instr_lsts: &[InstrLst]) -> HashMap<String, Vec<&InstrLst>> {
    let mut instr_lst_groups: HashMap<String, Vec<&InstrLst>> = HashMap::new();
    for instr_lst in instr_lsts {
        instr_lst_groups
            .entry(instr_lst.diag_addr.value.clone())
            .or_insert_with(Vec::new)
            .push(instr_lst);
    }
    instr_lst_groups
}

fn dedup_instr_lst_files(instr_lsts: &[&InstrLst]) -> Vec<String> {
    instr_lsts.iter()
        .map(|z| z.zdc_file.clone())
        .collect::<HashSet<_>>() // Deduplicate using HashSet
        .into_iter()
        .collect()
}

fn build_instr_lst_maps(instr_lsts: &[&InstrLst]) -> HashMap<(String, String), HashMap<String, Vec<String>>> {
    let mut parameter_map: HashMap<(String, String), HashMap<String, Vec<String>>> = HashMap::new();
    for instr_lst in instr_lsts {
        for instr in &instr_lst.lst {
            if let Some(service) = InstrLst::extract_service(instr) {
                if let Some(translations) = &service.transl {
                    if let Some(params) = &translations.params {
                        let srv_name = translations
                            .srv_name
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string());
                        for param in params {
                            parameter_map
                                .entry((srv_name.clone(), param.name.clone()))
                                .or_insert_with(HashMap::new)
                                .entry(param.value.clone())
                                .or_insert_with(Vec::new)
                                .push(service.phase.clone());
                        }
                    } else if let Some(data_sets) = &translations.data_sets {
                            let srv_name = translations
                            .srv_name
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string());
                        
                        for data_set in data_sets {
                            parameter_map
                                .entry((srv_name.clone(), data_set.srv_name.clone()))
                                .or_insert_with(HashMap::new)
                                .entry(data_set.value.clone())
                                .or_insert_with(Vec::new)
                                .push(service.phase.clone());
                        }

                    }
                
                
                } /*
                else if let Some(data_sets) = &service.data_sets {
                     for data_set in data_sets.data_set {
                        parameter_map
                            .entry((data_set.did.clone(), data_set.name.clone()))
                            .or_insert_with(HashMap::new)
                            .entry(data_set.name.clone())
                            .or_insert_with(Vec::new)
                            .push(service.phase.clone());
                    }
                    
                }  */
            }
        }
    }
    parameter_map
}

fn group_val_by_diag_addr(val: Option<&VehicleAnalysisLog>) /* -> HashMap<String, Section>*/ {
   
    
   /* let mut val_groups: HashMap<String, Section> = HashMap::new();  

    for section in val.unwrap().result.sections.iter() {
                
        let m = section.get_measurement_by_title(&"Identification".to_string()).unwrap();
        let undefined_value = "<undefined>".to_string();

        if let Some(value) = m.get_value_by_label(&"TABROW_SubsyIdent.Param_SubsyIdentID".to_string()) {
            let value_txt = value.get_value().unwrap_or(&undefined_value);
            println!("Section: {} -> {} -> {}", section.get_title().clone(), m.get_title(), value_txt);
        } else if let Some(value) = m.get_value_by_label(&"Gateway_Identification.Gateway_Identification_Diagnose_ID_VW".to_string()) {
            let value_txt = value.get_value().unwrap_or(&undefined_value);
            println!("Section: {} -> {} -> {}", section.get_title().clone(), m.get_title(), value_txt);
        } else if let Some(value) = m.get_value_by_label(&"TABROW_BusmaIdent.Param_ECUID".to_string()) {
            let value_txt = value.get_value().unwrap_or(&undefined_value);
            println!("Section: {} -> {} -> {}", section.get_title().clone(), m.get_title(), value_txt);
        } else if let Some(value) = m.get_value_by_label(&"System_Identification.System_Identification_Subsystem_ID".to_string()) {
            let value_txt = value.get_value().unwrap_or(&undefined_value);
            println!("Section: {} -> {} -> {}", section.get_title().clone(), m.get_title(), value_txt);
        } else if let Some(value) = m.get_value_by_label(&"System_Identification.System_Identification_Param_SubsyIdentID".to_string()) {
            let value_txt = value.get_value().unwrap_or(&undefined_value);
            println!("Section: {} -> {} -> {}", section.get_title().clone(), m.get_title(), value_txt);
        } else {
        println!("Section: {} -> {} -> {}", section.get_title().clone(), m.get_title(), undefined_value);
        }

     


        val_groups
            .entry(value_txt.clone())
            .or_insert_with(Vec::new)
            .push(section);
    } 
    val_groups
    } */
}

fn build_val_maps(val: Option<&VehicleAnalysisLog>) -> HashMap<(String, String), HashMap<String, Vec<String>>> {
    
    let mut val_map: HashMap<(String, String), HashMap<String, Vec<String>>> = HashMap::new();

    for section in val.unwrap().result.sections.iter() {
        let mut p0 = vec![];
        p0.push(section.get_title().clone());
        print_val_params(&mut p0, &section.get_measurements());
        p0.pop();
    }
    val_map
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
                println!("Nested!");
                print_val_params(p0, nested_measurements);
            }
            _ => (),
        }
        print_val_value(p0, &measurement.get_values());
        p0.pop();
    }
}    

 /*   
    let mut p0 = vec![];

    for section in val.result.sections.iter() {

    }
  */
    /* 
    for zdc in zdcs {
        for instr in &zdc.lst {
            if let Some(service) = Zdc::extract_service(instr) {
                if let Some(translations) = &service.transl {
                    if let Some(params) = &translations.params {
                        let srv_name = translations
                            .srv_name
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string());
                        for param in params {
                            parameter_map
                                .entry((srv_name.clone(), param.name.clone()))
                                .or_insert_with(HashMap::new)
                                .entry(param.value.clone())
                                .or_insert_with(Vec::new)
                                .push(service.phase.clone());
                        }
                    }
                }
            }
        }
    }*/


fn process_maps(
    diag_addr: &str,
    parameter_map: HashMap<(String, String), HashMap<String, Vec<String>>>,
    val: Option<&VehicleAnalysisLog>)
 {
    for ((srv_name, name), value_map) in parameter_map {
       
        let unique_values: Vec<_> = value_map.keys().collect();
        let mut all_phases = vec![];
        for phases in value_map.values() {
            all_phases.extend(phases.iter().cloned());
        }

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
                    let value = unique_values[0];
                    if value_map[value].len() > 1 {
                        println!(
                            "[ZDC-MATCH] [{}] >> {}.{}: {}",
                            all_phases.join(" -> "),
                            srv_name,
                            name,
                            value,                            
                        );
                    } else {
                        println!(
                            "[ZDC-SINGLE] [{}] >> {}.{}: {}",
                            all_phases.join(" -> "),
                            srv_name,
                            name,
                            value
                        );
                    }
                }
                _ => {
                    let all_values = unique_values
                        .iter()
                        .map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(" -> ");
                    println!(
                        "[ZDC-DIFF] [{}] >> {}.{}: {}",
                        all_phases.join(" -> "),
                        srv_name,
                        name,
                        all_values
                    );
                }
            },
            Some(ref target_val) => {
                let all_values = unique_values
                    .iter()
                    .map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                if unique_values.len() == 1 && unique_values[0] == target_val {
                    println!(
                        "[ZDC-VAL-MATCH] [{} => VAL] >> {}.{}: {} => {}",
                        all_phases.join(" -> "),
                        srv_name,
                        name,
                        all_values,                        
                        target_val
                    );
                } else {
                    println!(
                        "[ZDC-VAL-DIFF] [{} => VAL] >> {}.{}: {} => {}",
                        all_phases.join(" -> "),
                        srv_name,
                        name,
                        all_values,                        
                        target_val
                    );
                }
            }
        }
    }
}

fn compare_instr_lst_with_val(
    instr_lsts: &[InstrLst],
    val: Option<&VehicleAnalysisLog>)
    {
    
    let instr_lst_groups = group_instr_lst_by_diag_addr(instr_lsts);

    for (diag_addr, instr_lsts) in instr_lst_groups {
 
        // Build zdc parameter 
        let instr_lst_param_map = build_instr_lst_maps(&instr_lsts);
        
        if !instr_lst_param_map.is_empty() {
               // Deduplicate zdc_files
               let unique_files = dedup_instr_lst_files(&instr_lsts);

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
        let val_param_map = build_val_maps(Some(val));
      
        group_val_by_diag_addr(Some(val)); /* -> HashMap<String, Section>*/ 


        // Handle the case where the zip argument is provided
        compare_instr_lst_with_val(
            &instr_lsts, 
            Some(&val));

    } else {
        
        // Handle the case where the zip argument is not provided
        println!("None");
        compare_instr_lst_with_val(
            &instr_lsts,
            None);
    }

  
    Ok(())
}

