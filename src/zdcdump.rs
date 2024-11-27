use anyhow::Result;
use piwis_zdc::{Zdc};



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
fn dump_parameters(zdc: &Zdc, zdc_dump_config: &ZdcDumpConfig) {

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

pub fn zdcdump(args: &ZdcDumpArgs) -> Result<()> {
    
    let zdc_dump_config = &mut ZdcDumpConfig::new(args.include_zdc_file,
        args.include_diag_addr,
        args.include_phase,
        args.include_srv_name,
        args.compare_phases);

    let zdc_vec = &Zdc::from_dir(&args.dir)?;


    for instr in zdc_vec {
        dump_parameters(&instr, zdc_dump_config);
    }

    
    Ok(())
}