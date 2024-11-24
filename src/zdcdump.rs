use anyhow::Result;
use piwis_zdc::{HumanTranslations, Zdc, Instr};
use piwis_val::{Measurement, ValueEnum, VehicleAnalysisLog};


#[derive(clap::Args, Debug)]
pub struct ZdcDumpArgs {
    dir: String,
}
/*
fn print_translation(p0: &mut Vec<String>, m: &Translation) {
    if let Some(values) = &m.get_values() {
        for value in *values {
            p0.push(value.get_text().clone());
            println!("{}: {}", p0.join(" >> "), value.get_value().unwrap_or(&"undefined".to_string()));
            p0.pop();
        }
    }
}*/

fn print_transl_params(p0: &mut Vec<String>, t: &Vec<HumanTranslations>) {
    for translation in t {
       // p0.push(translation.params.clone());
     //   println!("{}: {}", p0.join(" >> "), translation.txt_value);
       // p0.pop();
    }
}

pub fn zdcdump(args: &ZdcDumpArgs) -> Result<()> {
    
    let zdc = &Zdc::from_dir(&args.dir)?;
    
    let mut p0 = vec![];

    for instr in zdc.iter() {

        p0.push(instr.zdc_file.clone());
        p0.push(instr.diag_addr.value.clone());
        //println!("{}", p0.join(" >> "));
  
        for srv in instr.lst.iter() {
            if let Some(transl) = srv.get_transl() {
                if let Some(params) = transl.get_params() {
                    for param in params.iter() {
                        p0.push(param.name.clone());
                        p0.push(param.value.clone());
                        println!("{}", p0.join(" >> "));
                        p0.pop();
                        p0.pop();   
                    }
                }
             }
        } 
        p0.pop();
        p0.pop();
    }
    Ok(())
}

