use anyhow::Result;
use piwis_zdc::{HumanTranslations, InstrLst};

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
/*
fn print_transl_params(p0: &mut Vec<String>, t: &Vec<Transl>) {
    for translation in t {
        p0.push(translation.parameter_name.clone());
        println!("{}: {}", p0.join(" >> "), translation.value);
        p0.pop();
    }
}
 */
pub fn zdcdump(args: &ZdcDumpArgs) -> Result<()> {
    let instr = &InstrLst::from_dir(&args.dir)?;
    
    /*  let mut p0 = vec![];
  
    p0.push(instr.lst.zdc_file.clone());
    p0.push(instr.lst.diag_addr.value.clone());
    for hex_srv in instr.lst.lst.hex_srv.iter() {
        p0.push(hex_srv.phase.clone());
        println!("{}", p0.join(" >> "));
        //p0.push(hex_srv.transl.srv_name.clone());
        //print_transl_params(&mut p0, &hex_srv.transl.params);
        //p0.pop();
        p0.pop();
    }
    p0.pop();
    p0.pop();
{*/
    Ok(())
}