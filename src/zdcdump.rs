use anyhow::Result;
use piwis_zdc::{Translation, ZdcInstructionList};

#[derive(clap::Args, Debug)]
pub struct ZdcdumpArgs {
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
fn print_translations(p0: &mut Vec<String>, t: &Vec<Translation>) {
    for translation in t {
        p0.push(translation.parameter_name.clone());
        println!("{}: {}", p0.join(" >> "), translation.value);
        p0.pop();
    }
}
 */
pub fn zdcdump(args: &ZdcdumpArgs) -> Result<()> {
    let zdc = &ZdcInstructionList::from_directory(&args.dir)?;
    
    let mut p0 = vec![];
    /*
    for section in zdc.hex_service.human_translations.sections.iter() {*/
    p0.push(zdc.zdc_file.clone());
    p0.push(zdc.diagnosis_address.value.clone());
    for hex_service in zdc.hex_service.iter() {
        p0.push(hex_service.phase.clone());
        println!("{}", p0.join(" >> "));
        //p0.push(hex_service.human_translations.service_name.clone());
        //print_translations(&mut p0, &hex_service.human_translations.translation);
        //p0.pop();
        p0.pop();
    }
    p0.pop();
    p0.pop();

    Ok(())
}