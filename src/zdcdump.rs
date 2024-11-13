use anyhow::Result;
use piwis_zdc::{Measurement, ZdcSession};

#[derive(clap::Args, Debug)]
pub struct ZdcdumpArgs {
    dir: String,
}

fn print_measurement(p0: &mut Vec<String>, m: &Measurement) {
    if let Some(values) = &m.get_values() {
        for value in *values {
            p0.push(value.get_text().clone());
            println!("{}: {}", p0.join(" >> "), value.get_value().unwrap_or(&"undefined".to_string()));
            p0.pop();
        }
    }
}

fn print_measurements(p0: &mut Vec<String>, m: &Vec<Measurement>) {
    for measurement in m {
        p0.push(measurement.get_title().clone());
        if let Some(submeasurements) = measurement.get_submeasurements() {
            print_measurements(p0, submeasurements);
        }
        print_measurement(p0, measurement);
        p0.pop();
    }
}

pub fn zdcdump(args: &ZdcdumpArgs) -> Result<()> {
    let zdc = &ZdcSession::from_directory(&args.dir)?;

    let mut p0 = vec![];
    for section in zdc.hex_service.sections.iter() {
        p0.push(section.get_title().clone());
        print_measurements(&mut p0, &section.get_measurements());
        p0.pop();
    }

    Ok(())
}