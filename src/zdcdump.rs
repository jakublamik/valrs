use anyhow::Result;
use piwis_zdc::{InstrLst};



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

/// Dumps all `ParameterTranslation.value` by `ParameterTranslation.name` in the given Zdc.
fn dump_parameters(instr_lst: &InstrLst, zdc_dump_config: &ZdcDumpConfig) {

    // Process each instruction
    for instr in &instr_lst.lst {
        if let Some(service) = InstrLst::extract_service(instr) {              
            if let Some(translations) = &service.transl {
              
                if let Some(params) = &translations.params {
                    println!(
                        "Diagnosis Address : {} >> ZDC File: {}",
                        instr_lst.diag_addr.value,
                        instr_lst.zdc_file
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
                } else if let Some(data_sets) = &translations.data_sets {
                 /*   println!(
                        "Diagnosis Address : {} >> ZDC File: {}",
                        instr_lst.diag_addr.value,
                        instr_lst.zdc_file
                    ); 
                    let srv_name = translations.srv_name.clone().unwrap_or_else(|| "Unknown".to_string());                        */
                    
                   // let phase_id = format!("{}x{}", service.phase, translations.rd_id.clone().unwrap_or_else(|| "Unknown".to_string()));

                    for data_set in data_sets {
                    println!(
                        "[{}] >> {}.{}: {}",
                        service.phase,
                        data_set.rd_id,
                        data_set.srv_name,
                        data_set.value
                        );

                    }
                }
            }
        
        
        } else  if let Some(data_sets) = InstrLst::extract_data_sets(instr) {   
            for data_set in &data_sets.data_set {
                println!(
                    "[DATASET] >> {}: {}",
                    data_set.did,
                    data_set.name
                    );   
            }
        }
        
    }
}

pub fn zdcdump(args: &ZdcDumpArgs) -> Result<()> {
    
    let zdc_dump_config = &mut ZdcDumpConfig::new(args.include_zdc_file,
        args.include_diag_addr,
        args.include_phase,
        args.include_srv_name,
        args.compare_phases);

    let instr_lst = &InstrLst::from_dir(&args.dir)?;


    for instr in instr_lst {
        dump_parameters(&instr, zdc_dump_config);
    }

    
    Ok(())
}