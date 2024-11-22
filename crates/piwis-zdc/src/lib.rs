use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use anyhow::Context;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstrLst {
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum Instr {
    ShortNameService(ShortNameSrv),
    Warten(Wait),
    HexService(HexSrv),
    FlashSession(FlashSession),
    DataSets(DataSets),
    EcuDefinition(EcuDef),
    Service(Srv),
}

impl InstrLst {
    pub fn from_dir(directory: &str) -> anyhow::Result<Vec<InstrLst>> {
        let path = Path::new(directory);
        if !path.is_dir() {
            return Err(anyhow::anyhow!("Provided path is not a directory."));
        }
        
        let mut result = Vec::new();

        for entry in fs::read_dir(path)? {
            
            
            let entry = entry?;
            let file_path = entry.path();
            

            if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
//                if file_path.extension().and_then(|ext| ext.to_str()) == Some("xml") {
                if file_name.contains("IL") && file_path.extension().and_then(|ext| ext.to_str()) == Some("xml") {
                    println!("Processing file: {}", file_name);

                    let file = File::open(&file_path)?;
                    let reader = BufReader::new(file);
                    let instr_lst = &mut quick_xml::de::Deserializer::from_reader(reader);

                    let deserialized: InstrLst = serde_path_to_error::deserialize(instr_lst).context("Failed deserializing")?;
                    result.push(deserialized);
                }
            }
        }
        return Ok(result);
    }
/*
    pub fn get_hex_service_by_phase(&self, phase: &str) -> Option<&HexService> {
        self.hex_service.iter().find(|s| s.get_phase) == phase)
    }

    pub fn get_section_by_title(&self, title: &str) -> Option<&Section> {
        self.hex_service.human_translations.sections.iter().find(|s| s.get_title() == title)
    } */
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
pub struct ShortNameSrv {
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "@Phase")]
    pub phase: String,
    #[serde(rename = "@PhaseDetail")]
    pub phase_detail: Option<String>,
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
    #[serde(rename = "Response")]
    pub response: Option<Response>,
    #[serde(rename = "HumanTranslations")]
    pub transl: Option<Transl>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FlashSession {
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "@Phase")]
    pub phase: String,
    #[serde(rename = "Kommentar")]
    pub comment: Option<Comment>,
    #[serde(rename = "Request")]
    pub request: Request,
    #[serde(rename = "Response")]
    pub response: Option<Response>,

}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(rename = "@Value")]
    pub value: Option<String>,
    #[serde(rename = "Parameter")]
    param: Option<Vec<Parameter>>, 
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    #[serde(rename = "@Value")]
    pub value: Option<String>,
    #[serde(rename = "Parameter")]
    param: Option<Vec<Parameter>>, 
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
pub struct HexSrv {
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
    #[serde(rename = "Request")]
    pub req: String,
    #[serde(rename = "ExpectedValue")]
    pub exp_value: Option<String>,
    #[serde(rename = "Response")]
    pub re: Option<String>,
    #[serde(rename = "HumanTranslations")]
    pub transl: Option<Transl>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Transl {
    #[serde(rename = "@ServiceID")]
    pub srv_id: Option<String>,
    #[serde(rename = "@RDIdentifier")]
    pub rd_id: Option<String>,
    #[serde(rename = "@ServiceName")]
    pub srv_name: Option<String>,
     #[serde(rename = "DatasetTranslation")]
    pub dataset_transl: Option<Vec<DatasetTransl>>,
    #[serde(rename = "Translation")]
    pub params: Option<Vec<Params>>,

}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DatasetTransl {
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
pub struct Params {
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
    pub neg_respones: NegResponses, 
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
    pub short_name_srv: Option<InstructionShortNameSrv>,
    #[serde(rename = "HexService")]
    pub hex_srv: Option<String>,
    #[serde(rename = "Wait")]
    pub wait: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InstructionShortNameSrv {
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
    pub zdc_value: ZdcValue,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ZdcValue {
    #[serde(rename = "$text")]
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
    pub flash_srv: Option<FlashSrv>,
    #[serde(rename = "ReadServices")]
    pub read_srvs: ReadSrvs
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PostModeInstructions {
    #[serde(rename = "PostModeInstruction")]
    pub instr: Vec<Instruction>,
}
/*
#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PostModeInstruction {
    #[serde(rename = "@Mode")]
    pub mode: String,
    #[serde(rename = "@IsWriteInstruction")]
    pub is_wrt_inst: String,
    #[serde(rename = "ShortNameService")]
    pub short_name_srv: Option<InstructionShortNameSrv>,
    #[serde(rename = "Wait")]
    pub wait: Option<String>,
}
 */
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
    pub read: FlashJobRe,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FlashJobRe {
    #[serde(rename = "@ShortName")]
    pub short_name: String,
    #[serde(rename = "Parameters")]
    pub params: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FlashSrv {
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
    pub re: CodingResponses,    
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
pub struct ReadSrvs {
    #[serde(rename = "ReadService")]
    pub read_service: ReadSrv,    
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ReadSrv {
    #[serde(rename = "@WriteServiceID")]
    pub wrt_srv_id: String,    
    #[serde(rename = "@WriteLocalID")]
    pub wrt_local_id: String,    
    #[serde(rename = "$text")]
    value: String,    
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NegResponses {
    #[serde(rename = "NegativeResponse")]
    pub respones: NegResponse, 
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NegResponse {
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
    CalcIVD(ProtectionRequest),
    ReadSlaveListConfigHash(ProtectionRequest),
    readSFD_ARS(ProtectionRequest),
    StartSFD_E2E(ProtectionRequest),
    EndSFD_E2E(ProtectionRequest),
    StatusSFD(ProtectionRequest),
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