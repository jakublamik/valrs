use anyhow::Result;
use piwis_val::{Measurement, VehicleAnalysisLog};
use piwis_zdc::{InstrLst};
use indexmap::{IndexMap, IndexSet};
use serde::de::value;
use std::collections::{HashMap, HashSet};
use regex::Regex;

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

fn strip_units(input: &str) -> String {
    // Define the sequences to strip, ordered from longest to shortest
    let units = [
        "m/s", "km/h", "l/100 km", "mpg US", "mpg UK", "mpg \\(US\\)", "mpg \\(UK\\)", "%/sec", "%/s", "km/l", "dB\\(A\\)", "ms", "km", "min", "mm", "\\?", "l", "\\(A\\)", "%", "°C", "g", "bar", "s", "kWh", "ml", "Ω"
    ];

    // Create a regex to match a number (integer or decimal, positive or negative)
    let number_regex = Regex::new(r"^-?\d+(\.\d+)?$").unwrap();

    // Check if the input contains a number followed by a unit
    if number_regex.is_match(&input.chars().take_while(|c| c.is_digit(10) || *c == '.' || *c == '-').collect::<String>()) {
        // Strip the units from the input
        let mut result = input.to_string();
        for seq in &units {
            let re = Regex::new(&format!(r"{}", seq)).unwrap();
            result = re.replace_all(&result, "").to_string();
        }
        result.trim().to_string()
    } else {
        // Return the input unchanged if it doesn't match the pattern
        input.to_string()
    }
}

fn get_val_value(val_map: HashMap<String, HashMap<String, String>>, diag_addr: &str, target_name: &str) -> Option<String> {
    
    let diag_addr = diag_addr.trim_start_matches('0').to_lowercase();
    
    let value_map = val_map.get(&diag_addr); 
 
    match value_map {
        Some(value_map) => {
            let target_value = value_map.get(target_name);
            return target_value.cloned();
        }
        None => {
         
            let value_map = val_map.get(&"<undefined>".to_string());           
            match value_map {
                Some(value_map) => {         
                    let target_value = value_map.get(target_name);                    
                    match target_value {
                        Some(target_value) => {         
                            Some(target_value.clone())
                        }
                        
                        None => {         
                            None
                        }
                    }
                }
                None => None,
            }
        },
    }
}

fn group_zdc_by_diag_addr(instr_lsts: &[InstrLst]) -> IndexMap<String, Vec<&InstrLst>> {
    let mut instr_lst_groups: IndexMap<String, Vec<&InstrLst>> = IndexMap::new();
    for instr_lst in instr_lsts {
        instr_lst_groups
            .entry(instr_lst.diag_addr.value.clone())
            .or_insert_with(Vec::new)
            .push(instr_lst);
    }
    instr_lst_groups
}

fn dedup_zdc_files(instr_lsts: &[&InstrLst]) -> Vec<String> {
    instr_lsts.iter()
        .map(|z| z.zdc_file.clone())
        .collect::<IndexSet<_>>() // Deduplicate using IndexSet
        .into_iter()
        .collect()
}

fn build_zdc_map(instr_lsts: &[&InstrLst]) -> IndexMap<(String, String), Vec<(String, String)>> {
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
    val_map: Option<HashMap<String, HashMap<String, String>>>)
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

        let target_value = match &val_map {           
            Some(val_map) => get_val_value(val_map.clone(), diag_addr, &target_name),
            None => None,
        };

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
               
                let stripped_unqiue_value = strip_units(&unique_values[0]);
                let stripped_target_value = strip_units(&target_val);
               
                if unique_values.len() == 1 && stripped_unqiue_value == *stripped_target_value { 
                //if unique_values.len() == 1 && unique_values[0] == *target_val {
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

fn compare_zdc_val(
    zdc_instr_lsts: &[InstrLst],
    val_map: Option<HashMap<String, HashMap<String, String>>>)
    {
    
    let zdc_groups = group_zdc_by_diag_addr(zdc_instr_lsts);

    for (diag_addr, instr_lsts) in zdc_groups {
 
        // Build zdc parameter 
        let zdc_map = build_zdc_map(&instr_lsts);
        
        if !zdc_map.is_empty() {
               // Deduplicate zdc_files
               let unique_files = dedup_zdc_files(&instr_lsts);

               // Print diag_addr and deduplicated zdc_files
               println!(
                   "Diagnosis Address: {} >> ZDC: {}",
                   diag_addr,
                   unique_files.join(",")
               );
            
            process_maps(&diag_addr, zdc_map, val_map.clone());
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
      
        let val_map =  build_val_map(Some(val)); /* -> IndexMap<String, Section>*/ 

        // Handle the case where the zip argument is provided
        compare_zdc_val(
            &instr_lsts, 
            Some(val_map));

    } else {
        
        // Handle the case where the zip argument is not provided
        compare_zdc_val(
            &instr_lsts,
            None);
    }

  
    Ok(())
}

fn build_val_map(val: Option<&VehicleAnalysisLog>) -> HashMap<(String), HashMap<String, String>> {
         
    let mut val_map: HashMap<String, HashMap<String, String>> = HashMap::new();

    if let Some(vehicle_log) = val {
        for section in &vehicle_log.result.sections {

            let identification = &section.get_measurement_by_title(&"Identification".to_string()).unwrap();

            let diagr_addr = get_section_diagr_addr( &identification);
            
            if diagr_addr == "<undefined>" {
                println!("Diagr addr for [{}] not found!", section.get_title().clone());
            }
        
            let measurements  = section.get_measurements(); 
            
            for measurement in measurements {
                
                // Skipping nested measurements as Faults are not relevant
                
                let values = &measurement.get_values();

                match &values {
                    Some(values) => { 

                        for value in *values {
                            
                            let mut value_txt = value.get_value().unwrap_or(&"<undefined>".to_string()).clone(); 
                            let unit_txt = value.get_unit();
                            
                            match unit_txt {
                                Some(unit_txt) => value_txt = format!("{}{}", value_txt, unit_txt),
                                None => (),
                            }
                            val_map
                                .entry(diagr_addr.clone())
                                .or_insert_with(HashMap::new)
                                .insert(value.get_label().clone(), value_txt);
                        }
                    }
                    None=> (),
                }
            }
        }
    }

    val_map
}

fn get_section_diagr_addr(measurement: &Measurement) -> String {

    let labels = [
        "TABROW_SubsyIdent.Param_SubsyIdentID",
        "Gateway_Identification.Gateway_Identification_Diagnose_ID_VW",
        "TABROW_BusmaIdent.Param_ECUID",
        "System_Identification.System_Identification_Subsystem_ID",
        "System_Identification.System_Identification_Param_SubsyIdentID",
    ];

    let diag_addr = labels.iter()
        .filter_map(|label| measurement.get_value_by_label(&label.to_string()))
        .map(|value| value.get_value().unwrap_or(&"<undefined>".to_string()).to_string())
        .next()
        .unwrap_or_else(|| "<undefined>".to_string());
    diag_addr
}