use anyhow::Result;
use piwis_zdc::{HumanTranslations, Zdc, Instr};

#[derive(clap::Args, Debug)]
pub struct ZdcDumpArgs {
    dir: String,
}

fn print_params(p0: &mut Vec<String>, transl: &HumanTranslations) {
    
    if let Some(srv_name) = transl.get_srv_name() {
        p0.push(srv_name.clone()); // Push the name if it exists
    }
            
    if let Some(params) = transl.get_params() {
        for param in params.iter() {
            p0.push(param.name.clone());
            println!("{}: {}", p0.join(" >> "), param.value);
            p0.pop();
            
        }
    }

    if let Some(_srv_name) = transl.get_srv_name() {
        p0.pop();
    }
}
   

fn print_instr_transl(p0: &mut Vec<String>, instrs: &Vec<Instr>) {
   

       
    for instr in instrs.iter() {
          if let Some(phase) = instr.get_phase() {
            p0.push(phase.clone()); // Push the name if it exists
        }
        if let Some(transl) = instr.get_transl() {
            print_params(p0, &transl);
        }
        if let Some(_phase) = instr.get_phase() {
            p0.pop(); // Push the name if it exists
        }
    
    } 

}

pub fn zdcdump(args: &ZdcDumpArgs) -> Result<()> {
    
    let zdc = &Zdc::from_dir(&args.dir)?;
    
    let mut p0 = vec![];

    for instr in zdc.iter() {
        p0.push(instr.zdc_file.clone());
        p0.push(instr.diag_addr.value.clone());

        print_instr_transl(&mut p0, &instr.lst);

        //println!("{}", p0.join(" >> "));
  
   
        p0.pop();
        p0.pop();
    }
    Ok(())
}

