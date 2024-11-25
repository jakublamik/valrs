use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Zdc {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    #[serde(rename = "@ZDCFile")]
    pub zdc_file: String,
    #[serde(rename = "@DATEI-ID")]
    pub file_id: String,
    #[serde(rename = "@VERSION-INHALT")]
    pub ver_content: String,
    #[serde(rename = "diagnosisAddress")]
    pub diag_addr : DiagAddr,
    #[serde(rename="$value")]
    pub lst: Vec<Instr>,
}

impl Zdc {
    pub fn from_dir(directory: &str) -> anyhow::Result<Vec<Zdc>> {
        let path = Path::new(directory);
        if !path.is_dir() {
            return Err(anyhow::anyhow!("Provided path is not a directory."));
        }
        let mut result = Vec::new();
        for entry in fs::read_dir(path)? {         
            let entry = entry?;
            let file_path = entry.path();
            if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                if file_name.contains("IL") && file_path.extension().and_then(|ext| ext.to_str()) == Some("xml") {
                    println!("Deserializing: {}", file_name);
                    let file = File::open(&file_path)?;
                    let reader = BufReader::new(file);
                    let instr_lst = &mut quick_xml::de::Deserializer::from_reader(reader);
                    let deserialized: Zdc = serde_path_to_error::deserialize(instr_lst).context("Failed deserializing")?;
                    result.push(deserialized);
                }
            }
        }
        return Ok(result);
    }

    
    pub fn compare_parameters(&self) {
        println!("Processing Zdc with diag_addr: {}", self.diag_addr.value);

        // Map to track `ParameterTranslation` by name
        let mut parameter_map: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

        // Process each instruction
        for instr in &self.lst {
            if let Some(service) = Self::extract_service(instr) {
                if let Some(translations) = &service.transl {
                    if let Some(params) = &translations.params {
                        for param in params {
                            parameter_map
                                .entry(param.name.clone())
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
        for (name, value_map) in parameter_map {
            let unique_values: Vec<_> = value_map.keys().collect();
            match unique_values.len() {
                1 => {
                    // Only one unique value
                    let value = unique_values[0];
                    let phases = &value_map[value];
                    if phases.len() > 1 {
                        println!(
                            "[MATCHING] Name: {}, Value: {}, Phases: {:?}",
                            name, value, phases
                        );
                    } else {
                        println!(
                            "[NEW] Name: {}, Value: {}, Phases: {:?}",
                            name, value, phases
                        );
                    }
                }
                _ => {
                    // Multiple unique values exist
                    let mut all_values = vec![];
                    let mut all_phases = vec![];
                    for (value, phases) in value_map {
                        all_values.push(value);
                        all_phases.extend(phases);
                    }
                    println!(
                        "[DIFFERENT] Name: {}, Values: {:?}, Phases: {:?}",
                        name, all_values, all_phases
                    );
                }
            }
        }
    }

    /// Extracts the `Service` object from an `Instr` if it contains one.
    fn extract_service(instr: &Instr) -> Option<&Service> {
        match instr {
            Instr::ShortNameService(service) => Some(service),
            Instr::HexService(service) => Some(service),
            Instr::FlashSession(service) => Some(service),
            _ => None, // Other types of Instr do not contain a Service
        }
    }
    
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagAddr {
    #[serde(rename = "@codingOrder")]
    pub coding_order: String,
    #[serde(rename = "@IVD")]
    pub ivd: Option<String>,
    #[serde(rename = "@SFD")]
    pub sfd: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum Instr {
    ShortNameService(Service),
    Warten(Wait),
    HexService(Service),
    FlashSession(Service),
    DataSets(DataSets),
    EcuDefinition(EcuDef),

}
impl Instr {
    pub fn get_transl(&self) -> Option<&HumanTranslations> {
        match self {
            Instr::ShortNameService(instr) => instr.transl.as_ref(),
            Instr::HexService(instr) => instr.transl.as_ref(),
            Instr::FlashSession(instr) => instr.transl.as_ref(),
            _ => None,
        }
    }
    pub fn get_phase(&self) -> Option<&String> {
        match self {
            Instr::ShortNameService(instr) => Some(&instr.phase),
            Instr::HexService(instr) => Some(&instr.phase),
            Instr::FlashSession(instr) => Some(&instr.phase),
            _ => None,
        }
    }
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "@Phase")]
    pub phase: String,
    #[serde(rename = "@PhaseDetail")]
    pub phase_detail: Option<String>,
    #[serde(rename = "@did")]
    pub did: Option<String>,  
    #[serde(rename = "@Mode")]
    pub mode: Option<String>,
    #[serde(rename = "@Bewertung")]
    pub eval: Option<String>,
    #[serde(rename = "Kommentar")]
    pub comment: Option<Comment>,
    #[serde(rename = "DataSets")]
    pub data_sets: Option<DataSets>,
    #[serde(rename = "Request")]
    pub request: Request,
    #[serde(rename = "ExpectedValue")]
    pub exp_value: Option<String>,
    #[serde(rename = "Response")]
    pub response: Option<Response>,
    #[serde(rename = "HumanTranslations")]
    pub transl: Option<HumanTranslations>,
}
/*
impl Service {
    pub fn find_parameter_translation(&self, name: &str) -> Option<&ParameterTranslation> {
        self.transl
            .as_ref()? // Check if `transl` exists
            .params
            .as_ref()? // Check if `params` exists
            .iter()
            .find(|param| param.name == name) // Find the first match
    }
}
*/
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(rename = "@Value")]
    pub value: Option<String>,
    #[serde(rename = "Parameter")]
    params: Option<Vec<Parameter>>,
    #[serde(rename = "$text")]
    txt_value: Option<String>,  
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    #[serde(rename = "@Value")]
    pub value: Option<String>,
    #[serde(rename = "Parameter")]
    param: Option<Vec<Parameter>>, 
    #[serde(rename = "$text")]
    txt_value: Option<String>,  
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    #[serde(rename = "@ShortName")]
    pub short_name: String,
    #[serde(rename = "@Value")]
    pub value: String,
    #[serde(rename = "@space")]
    pub xml_space: Option<String>, 
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Comment {
    #[serde(rename = "@space")]
    pub xml_space: String,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Wait {
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct HumanTranslations {
    #[serde(rename = "@ServiceID")]
    pub srv_id: Option<String>,
    #[serde(rename = "@RDIdentifier")]
    pub rd_id: Option<String>,
    #[serde(rename = "@ServiceName")]
    pub srv_name: Option<String>,
     #[serde(rename = "DatasetTranslation")]
    pub datasets: Option<Vec<DatasetTransalations>>,
    #[serde(rename = "Translation")]
    pub params: Option<Vec<ParameterTranslation>>,

}

impl HumanTranslations {
    pub fn get_params(&self) -> Option<&Vec<ParameterTranslation>> {
        self.params.as_ref()
    }
    pub fn get_srv_name(&self) -> Option<&String> {
        self.srv_name.as_ref()
    }
   /*
    pub fn get_param_by_name(&self, name: &str) -> Option<&ParameterTranslation> {
        self.params.iter().find(|s| s.get_name() == name)
    } */  
}


#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ParameterTranslation {
    #[serde(rename = "@ParameterName")]
    pub name: String,
    #[serde(rename = "@BytePosition")]
    pub byte_pos: String,
    #[serde(rename = "@LSB")]
    pub lsb: String,
    #[serde(rename = "@BitLength")]
    pub bit_len: String,
    #[serde(rename = "@HexValue")]
    pub hex_value: String,
    #[serde(rename = "$text")]
    pub value: String,
}

impl Clone for ParameterTranslation {
    fn clone(&self) -> Self {
        ParameterTranslation {
            name: self.name.clone(),
            byte_pos: self.byte_pos.clone(),
            lsb: self.lsb.clone(),
            bit_len: self.bit_len.clone(),           
            hex_value: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

/*
impl ParameterTranslation {
    pub fn get_name(&self) -> &String {
        &self.name
    }   
}
 */
#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DatasetTransalations{
    #[serde(rename = "@RDIdentifier")]
    pub rd_id: String,
    #[serde(rename = "@ServiceName")]
    pub srv_name: String,
    #[serde(rename = "@HexValue")]
    pub hex_value: String,
    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DataSets {
    #[serde(rename = "DataSet")]
    pub data_set: Vec<DataSet>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DataSet {
    #[serde(rename = "@did")]
    pub did: String,
    #[serde(rename = "@Bewertung")]
    pub eval: Option<String>,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "value")]
    pub value: Option<String>,
    #[serde(rename = "Kommentar")]
    pub comment: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EcuDef {
    #[serde(rename = "@AlwaysCoding")]
    pub always_coding: String,
    #[serde(rename = "@AlwaysDSDL")]
    pub always_dsl: String,
    #[serde(rename = "@CreateDSDLControlFile")]
    pub create_ctrl_file: String,
    #[serde(rename = "@AlwaysReadAll")]
    pub always_rd_all: String,
    #[serde(rename = "@UseDSName")]
    pub use_ds_name: String, 
    #[serde(rename = "@DSCheck")]
    pub ds_check: String,
    #[serde(rename = "@ECUName")]
    pub ecu_name: Option<String>,
    #[serde(rename = "@DiagAddress")]
    pub diag_addr: Option<String>,
    #[serde(rename = "@diagnosisClass")]
    pub diag_class: Option<String>,
    #[serde(rename = "@nodeAddress")]
    pub node_addr: Option<String>,
    #[serde(rename = "@diagnosisAddressMaster")]
    pub diag_addr_master: Option<String>,
    #[serde(rename = "PreCodingInstructions")]
    pub pre: Instructions, 
    #[serde(rename = "Coding")]
    pub coding: Coding, 
    #[serde(rename = "PostCodingInstructions")]
    pub post: Instructions, 
    #[serde(rename = "NegativeResponses")]
    pub neg_respones: NegativeResponses, 
    #[serde(rename = "IVD")]
    pub ivd: Vec<VehicleProtection>,
    #[serde(rename = "SFD")]
    pub sfd: Vec<VehicleProtection>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Instructions {
    #[serde(rename = "Instruction")]
    pub instr: Vec<Instruction>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Instruction {
    #[serde(rename = "@Mode")]
    pub mode: Option<String>,
    #[serde(rename = "@IsWriteInstruction")]
    pub is_wrt_inst: String,
    #[serde(rename = "ShortNameService")]
    pub short_name_srv: Option<InstructionShortNameService>,
    #[serde(rename = "HexService")]
    pub hex_srv: Option<String>,
    #[serde(rename = "Wait")]
    pub wait: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InstructionShortNameService {
    #[serde(rename = "@ShortName")]
    pub short_name: String,
    #[serde(rename = "Parameters")]
    pub params: Option<PrePostParams>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PrePostParams {
    #[serde(rename = "Parameter")]
    pub param: Option<PrePostParam>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PrePostParam {
    #[serde(rename = "@ShortName")]
    pub short_name: String,
    #[serde(rename = "ZDCValue")]
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Coding {
    #[serde(rename = "PostModeInstructions")]
    pub post_mode_instr: Option<PostModeInstructions>,
    #[serde(rename = "FlashJob")]
    pub flash_job: Option<FlashJob>,
    #[serde(rename = "FlashService")]
    pub flash_srv: Option<FlashService>,
    #[serde(rename = "ReadServices")]
    pub read_srvs: ReadServices,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PostModeInstructions {
    #[serde(rename = "PostModeInstruction")]
    pub instr: Vec<Instruction>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FlashJob {
    #[serde(rename = "@ShortName")]
    pub short_name: String,
    #[serde(rename = "@SessionShortName")]
    pub session_short_name: Option<String>,
    #[serde(rename = "@ParameterNameControlFile")]
    pub param_name_ctrl_file: String,
    #[serde(rename = "Parameters")]
    pub params: Option<PrePostParams>,
    #[serde(rename = "Responses")]
    pub responses: CodingResponses,
    #[serde(rename = "ReadService")]
    pub read: ReadService,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FlashService {
    #[serde(rename = "@ShortName")]
    pub short_name: String,
    #[serde(rename = "@ParameterNameData")]
    pub param_name_data: String,
    #[serde(rename = "@ParameterNameSize")]
    pub param_name_size: String,
    #[serde(rename = "@ParameterNameStartAddress")]
    pub param_name_str_addr: String,
    #[serde(rename = "@ParameterNameKodiercontainerPartNumber")]
    pub param_name_container_pn: String,
    #[serde(rename = "@ParameterNameKodiercontainerVersion")]
    pub param_name_container_ver: String,
    #[serde(rename = "Responses")]
    pub responses: CodingResponses,    
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CodingResponses {
    #[serde(rename = "Response")]
    pub response: Vec<CodingResponse>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CodingResponse {
    #[serde(rename = "@ShortName")]
    pub short_name: String,
    #[serde(rename = "@Rating")]
    pub rating: String,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ReadServices {
    #[serde(rename = "ReadService")]
    pub read_service: ReadService,    
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ReadService {
    #[serde(rename = "@ShortName")]
    pub short_name: Option<String>,
    #[serde(rename = "Parameters")]
    pub params: Option<String>,
    #[serde(rename = "@WriteServiceID")]
    pub wrt_srv_id: Option<String>,    
    #[serde(rename = "@WriteLocalID")]
    pub wrt_local_id: Option<String>,    
    #[serde(rename = "$text")]
    value: Option<String>,    
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NegativeResponses {
    #[serde(rename = "NegativeResponse")]
    pub respones: NegativeResponse, 
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NegativeResponse {
    #[serde(rename = "@ResponseCode")]
    pub code: String,
    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct VehicleProtection {
    #[serde(rename="$value")]
    pub veh_prot: Vec<VehicleProtectionOps>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum VehicleProtectionOps {
    #[serde(rename="CalcIVD")]
    CalcIvd(ProtectionRequest),
    #[serde(rename="ReadSlaveListConfigHash")]
    ReadSlaveListConfigHash(ProtectionRequest),
    #[serde(rename="readSFD_ARS")]
    ReadSfdArs(ProtectionRequest),
    #[serde(rename="StartSFD_E2E")]
    StartSfdE2e(ProtectionRequest),
    #[serde(rename="EndSFD_E2E")]
    EndSdfE2e(ProtectionRequest),
    #[serde(rename="StatusSFD")]
    StatusSfd(ProtectionRequest),
    #[serde(rename="ModeOfProtection")]
    ModeOfProtection(ProtectionRequest),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectionRequest {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "Request")]
    pub request: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Srv {
    #[serde(rename = "RawData")]
    pub raw_data: RawData,

}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawData {
    #[serde(rename = "@Type")]
    pub raw_data_type: String,
    #[serde(rename = "$text")]
    pub value: String,

}

