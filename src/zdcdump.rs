use anyhow::Result;
use piwis_zdc::{HumanTranslations, Zdc, Instr};
use std::collections::HashMap;
use itertools::Itertools;

#[derive(clap::Args, Debug)]
pub struct ZdcDumpArgs {
    dir: String,
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
pub struct ZdcDumpConfig {
    pub(crate) include_zdc_file: bool,
    pub(crate) include_diag_addr: bool,
    pub(crate) include_phase: bool,
    pub(crate) include_srv_name: bool,
    pub(crate) compare_phases: bool,
}

impl ZdcDumpConfig {
    pub fn new(include_zdc_file: bool, include_diag_addr: bool, include_phase: bool, include_srv_name: bool, compare_phases: bool) -> ZdcDumpConfig {
        ZdcDumpConfig {
            include_zdc_file,
            include_diag_addr,
            include_phase,
            include_srv_name,
            compare_phases,
        }
    }
}
/*
pub fn get_target_value_by_name(diag_addr: &str, target_name: &str) ->  Option<String> {
    
    match (diag_addr, target_name) {
        ("003C", "CodingValue.Bitfield_Param_RearAxleSteer") => Some("with_Rear_Axle_Steering".to_string()),
        ("0069", "CodingValue.Param_StatuTermi30OutpuPin9") => Some("activated_trailer_mode".to_string()),
        ("0069", "CodingValue.Bitfield2_Param_DimmuFLED") => Some("activated_trailer_mode".to_string()),
        ("0649", "CodingValue.Bitfield_Param_DiagnConneA") => Some("0A0100".to_string()),
        ("064E", "TABROW_RearLidPowerOpeniContrUnitDataSetUploa.Param_EasyOpenHubCompa") => Some("43".to_string()),
        _ => None,
    }
}*/

pub fn get_target_value_by_name(diag_addr: &str, target_name: &str) -> Option<String> {
    // Example: Predefined mapping of diag_addr and target_name to target values
    let predefined_values: HashMap<(&str, &str), String> = HashMap::from([
        (("003C", "CodingValue.Bitfield_Param_RearAxleSteer"), "with_Rear_Axle_Steerin".to_string()),
        (("0069", "CodingValue.Param_StatuTermi30OutpuPin9"), "activated_trailer_mode".to_string()),
        (("0069", "CodingValue.Bitfield2_Param_DimmuFLED"), "activated_trailer_mode".to_string()),
        (("0036", "CodingValue.Bitfield_Param_VehicWithoPasseSSG"), "passenger_SSG_installed".to_string()),
        (("0036", "CodingValue.Bitfield2_Param_LocatLordoButto"), "not_active".to_string()),
        ]);

        // Attempt to find the target value in the map
        predefined_values.get(&(diag_addr, target_name)).cloned()
    }

pub fn zdcdump(args: &ZdcDumpArgs) -> Result<()> {
    
    let zdc_dump_config = &mut ZdcDumpConfig::new(args.include_zdc_file,
        args.include_diag_addr,
        args.include_phase,
        args.include_srv_name,
        args.compare_phases);

    let zdc_vec = &Zdc::from_dir(&args.dir)?;

    compare_parameters_with_target_in_all_zdcs(
        &zdc_vec,
        &|diag_addr, target_name| get_target_value_by_name(diag_addr, target_name),
    );
    /*
    for instr in zdc_vec {
        compare_parameters(&instr, zdc_dump_config);
    }
    */
    
    /*
    // Compare target value with parameters in Zdc entries
    compare_target_with_zdc_parameters(
        &zdc_vec,
        "0006",
        "CodingValue.Bitfield2_Param_SLVButtoLocat",
        "internal_contro2l",
    );
    */

    /*
    // Find all Zdc instances with matching diag_addr.value
    let matching_zdcs = find_zdcs_by_diag_addr(&zdc_vec, "0069");

    if !matching_zdcs.is_empty() {
        println!("Found {} Zdc(s) with diag_addr: {ADDR1}", matching_zdcs.len());

        // Now, find the ParameterTranslation value within the matching Zdc entries
        if let Some(value) = find_parameter_translation_value_in_zdcs(&matching_zdcs, "CodingValue.Param_Switc") {
            println!("Found ParameterTranslation.value: {}", value);
        } else {
            println!("ParameterTranslation not found.");
        }
    } else {
        println!("No Zdc found with the given diag_addr.value.");
    } 
    */
    

    /*
     let mut p0 = vec![];

    for instr in zdc_vec.iter() {
        p0.push(instr.diag_addr.value.clone());
        p0.push(instr.zdc_file.clone());
 
        
        print_instr_transl(&mut p0, &instr.lst);     

        p0.pop();
        p0.pop();
    }

    */

    Ok(())
}

    /// Dumps all `ParameterTranslation.value` by `ParameterTranslation.name` in the given Zdc.
    pub fn dump_parameters(zdc: &Zdc, zdc_dump_config: &ZdcDumpConfig) {

        // Process each instruction
        for instr in &zdc.lst {
            if let Some(service) = Zdc::extract_service(instr) {              
                if let Some(translations) = &service.transl {
                    if let Some(params) = &translations.params {
                        println!(
                            "Diagnosis Address : {} >> ZDC File: {}",
                            zdc.diag_addr.value,
                            zdc.zdc_file
                        );
                        let srv_name = translations.srv_name.clone().unwrap_or_else(|| "Unknown".to_string());                        
                        for param in params {
                        println!(
                            "[{}] >> {}.{}: {}",
                            service.phase,
                            srv_name,
                            param.name,
                            param.value
                            );

                        }
                    }
                }
            }
        }
    }

    /// Compares `ParameterTranslation.value` by `ParameterTranslation.name` in the given Zdc.
    pub fn compare_parameters(zdc: &Zdc, zdc_dump_config: &ZdcDumpConfig) {
        let mut printed_diag_addr = false;

        // Map to track each combination of `srv_name`, `name`, and `value`
        let mut parameter_map: HashMap<(String, String), HashMap<String, Vec<String>>> = HashMap::new();

        // Process each instruction
        for instr in &zdc.lst {
            if let Some(service) = Zdc::extract_service(instr) {
                if let Some(translations) = &service.transl {
                    if let Some(params) = &translations.params {
                        let srv_name = translations.srv_name.clone().unwrap_or_else(|| "Unknown".to_string());
                        for param in params {
                            // Create a unique key based on srv_name and name
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

        // Analyze and classify results
        for ((srv_name, name), value_map) in parameter_map {
            if !printed_diag_addr {
                // Print diag_addr only if there's a valid output
                println!(
                    "Diagnosis Address : {} >> ZDC File: {}",
                    zdc.diag_addr.value,
                    zdc.zdc_file
                );
                printed_diag_addr = true;
            }

            let unique_values: Vec<_> = value_map.keys().collect();

            // Collect all phases across values for consistency in formatting
            let mut all_phases = vec![];
            for phases in value_map.values() {
                all_phases.extend(phases.iter().cloned());
            }

            match unique_values.len() {
                1 => {
                    // Only one unique value
                    let value = unique_values[0];
                    let srv_names_phases = &value_map[value];
                    if srv_names_phases.len() > 1 {
                        println!(
                            "[ZDC MATCH] [{}] >> {}.{}: {}",
                            all_phases.join(", "),
                            srv_name,
                            name,
                            value
                        );
                    } else {
                        println!(
                            "[ZDC SINGLE] [{}] >> {}.{}: {}",
                            srv_names_phases[0], // Phase
                            srv_name,
                            name,
                            value
                        );
                    }
                }
                _ => {
                    // Multiple unique values exist
                    let all_values = unique_values
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(" <-> ");
                    println!(
                        "[ZDC CHANGE] [{}] >> {}.{}: {}",
                        all_phases.join(", "),
                        srv_name,
                        name,
                        all_values
                    );
                }
            }
        }
    }



/// Finds all `Zdc` instances with the matching `diag_addr.value`.
pub fn find_zdcs_by_diag_addr<'a>(zdc_list: &'a [Zdc], target_addr: &str) -> Vec<&'a Zdc> {
    zdc_list.iter().filter(|zdc| zdc.diag_addr.value == target_addr).collect()
}

/// Finds the `ParameterTranslation.value` for the given `name.srv_name` across multiple `Zdc`s.
pub fn find_parameter_translation_value_in_zdcs<'a>(
    zdcs: &[&'a Zdc],
    target_name: &str,
) -> Option<String> {
    for zdc in zdcs {
        if let Some(value) = zdc.find_parameter_translation_value(target_name) {
            return Some(value);
        }
    }
    None
}

// Compare target value with parameters in Zdc entries
pub fn compare_target_with_zdc_parameters(
    zdc_list: &[Zdc],
    diag_addr_value: &str,
    target_name: &str,
    target_value: &str,
) {
    // Find all Zdc instances matching the specified `diag_addr_value`
    let matching_zdcs = find_zdcs_by_diag_addr(zdc_list, diag_addr_value);

    if matching_zdcs.is_empty() {
        println!("No Zdc found with the given diag_addr.value: {}", diag_addr_value);
        return;
    }

    // Collect all matching parameter values and phases
    let mut parameter_values: Vec<(String, String)> = vec![]; // (value, phase)
    let mut srv_name = String::new();

    for zdc in &matching_zdcs {
        for instr in &zdc.lst {
            if let Some(service) = Zdc::extract_service(instr) {
                if let Some(translations) = &service.transl {
                    if let Some(params) = &translations.params {
                        srv_name = translations.srv_name.clone().unwrap_or_else(|| "Unknown".to_string());
                        for param in params {
                            let full_name = format!("{}.{}", srv_name, param.name);
                            if full_name == target_name {
                                parameter_values.push((param.value.clone(), service.phase.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    if parameter_values.is_empty() {
        println!("No ParameterTranslation found with name: {}", target_name);
        return;
    }

    // Process and compare values
    let phases: Vec<String> = parameter_values.iter().map(|(_, phase)| phase.clone()).collect();
    let zdc_values: Vec<String> = parameter_values.iter().map(|(value, _)| value.clone()).collect();

    if zdc_values.iter().all(|value| value == target_value) && zdc_values.len() > 0 {
        // All values match the target
        println!(
            "[ZDC/VAL MATCH] [{}] >> {}: {}",
            phases.join(", "),
            target_name,
            target_value
        );
    } else {
        // At least one value differs
        let zdc_value_str = zdc_values.join(" <-> ");
        println!(
            "[ZDC/VAL DIFF] [{}] >> {}: {} <=> {}",
            phases.join(", "),
            target_name,
            zdc_value_str,
            target_value
        );
    }
}

pub fn compare_parameters_with_target_in_all_zdcs(
    zdcs: &[Zdc],
    get_target_value_by_name: &dyn Fn(&str, &str) -> Option<String>,
) {
    let mut zdc_groups: HashMap<String, Vec<&Zdc>> = HashMap::new();

    // Group ZDCs by `diag_addr.value`
    for zdc in zdcs {
        zdc_groups
            .entry(zdc.diag_addr.value.clone())
            .or_insert_with(Vec::new)
            .push(zdc);
    }

    for (diag_addr, zdcs) in zdc_groups {
        println!(
            "Diagnosis Address : {} >> ZDC Files: {}",
            diag_addr,
            zdcs.iter()
                .map(|z| &z.zdc_file)
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        );

        let mut parameter_map: HashMap<(String, String), HashMap<String, Vec<String>>> = HashMap::new();

        // Build parameter map
        for zdc in &zdcs {
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
        }
        for ((srv_name, name), value_map) in parameter_map {
            let unique_values: Vec<_> = value_map.keys().collect();
        
            let mut all_phases = vec![];
            for phases in value_map.values() {
                all_phases.extend(phases.iter().cloned());
            }
        
            let target_name = format!("{}.{}", srv_name, name);
            let target_value = get_target_value_by_name(&diag_addr, &target_name);
        
            match target_value {
                None => match unique_values.len() {
                    1 => {
                        let value = unique_values[0];
                        if value_map[value].len() > 1 {
                            println!(
                                "[ZDC MATCH] [{}] >> {}.{}: {}, {}",
                                all_phases.join(", "),
                                srv_name,
                                name,
                                unique_values
                                    .iter() // Use `iter` to avoid moving `unique_values`
                                    .map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                    unique_values
                                    .iter() // Use `iter` to avoid moving `unique_values`
                                    .map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")

                            );
                        } else {
                            println!(
                                "[ZDC SINGLE] [{}] >> {}.{}: {}",
                                all_phases.join(", "),
                                srv_name,
                                name,
                                value
                            );
                        }
                    }
                    _ => {
                        let all_values = unique_values
                            .iter() // Use `iter` to avoid moving `unique_values`
                            .map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        println!(
                            "[ZDC DIFF] [{}] >> {}.{}: {}, {}",
                            all_phases.join(", "),
                            srv_name,
                            name,
                            all_values,
                            all_values
                        );
                    }
                },
                Some(ref target_val) => {
                    let all_values = unique_values
                        .iter() // Use `iter` to avoid moving `unique_values`
                        .map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    if unique_values.len() == 1 && unique_values[0] == target_val {
                        println!(
                            "[ZDC/VAL MATCH] [{}, VAL] >> {}.{}: {}, {}, {}",
                            all_phases.join(", "),
                            srv_name,
                            name,
                            all_values,
                            all_values,
                            target_val
                        );
                    } else {
                        println!(
                            "[ZDC/VAL DIFF] [{}, VAL] >> {}.{}: {}, {}, {}",
                            all_phases.join(", "),
                            srv_name,
                            name,
                            all_values,
                            all_values,
                            target_val
                        );
                    }
                }
            }
        }
    }
}