use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use anyhow::Context;
use serde::{Deserialize, Deserializer, Serialize};
use serde_untagged::UntaggedEnumVisitor;

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ZdcSession {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    #[serde(rename = "@ZDCFile")]
    pub zdc_file: String,
    #[serde(rename = "@DATEI-ID")]
    pub datei_id: String,
    #[serde(rename = "@VERSION-INHALT")]
    pub version_inhalt: String,
    #[serde(rename = "diagnosisAddress")]
    pub diagnosis_address : DiagnosisAddress,
    #[serde(rename = "ShortNameService")]
    pub short_name_service : Vec<ShortNameService>,
    #[serde(rename = "Warten")]
    pub warten : Vec<Warten>,
    #[serde(rename = "HexService")]
    pub hex_service : HexService,
}

impl ZdcSession {
    pub fn from_directory(directory: &str) -> anyhow::Result<ZdcSession> {
        let path = Path::new(directory);
        if !path.is_dir() {
            return Err(anyhow::anyhow!("Provided path is not a directory."));
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
    
            // Check if the file has a .xml extension and contains "IExIL" in the filename
            if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                if file_name.contains("IExIL") && file_path.extension().and_then(|ext| ext.to_str()) == Some("xml") {
                    println!("Filename: {}", file_name);
                    // Open the file as BufReader
                    let file = File::open(&file_path)?;
                    let reader = BufReader::new(file);
                    let zdc = &mut quick_xml::de::Deserializer::from_reader(reader);
                    // Pass the BufReader to quick_xml for processing
                    let deserialized: ZdcSession = serde_path_to_error::deserialize(zdc).context("Failed deserializing")?;
                    return Ok(deserialized);
                }
            }
        }
        Err(anyhow::anyhow!("Could not find IExIL ZDC files in directory."))
    }

    pub fn get_section_by_title(&self, title: &str) -> Option<&Section> {
        self.hex_service.sections.iter().find(|s| s.get_title() == title)
    }
}



#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct DiagnosisAddress {
    #[serde(rename = "@codingOrder")]
    pub coding_order: String,
    #[serde(rename = "@IVD")]
    pub ivd: String,
    #[serde(rename = "@SFD")]
    pub sfd: String,
    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ShortNameService {
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "@Phase")]
    pub phase: String,
    #[serde(rename = "@PhaseDetail")]
    pub phase_detail: String,
    #[serde(rename = "@Mode")]
    pub mode: Option<String>,
    #[serde(rename = "@Bewertung")]
    pub bewertung: Option<String>,
    #[serde(rename = "Kommentar")]
    pub kommentar: Kommentar,
    #[serde(rename = "Request")]
    pub request: Request,
    #[serde(rename = "Response")]
    pub response: Option<Response>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Request {
    #[serde(rename = "@Value")]
    pub value: Option<String>,
    #[serde(rename = "Parameter")]
    parameter: Option<Vec<Parameter>>, 
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Response {
    #[serde(rename = "@Value")]
    pub value: Option<String>,
    #[serde(rename = "Parameter")]
    parameter: Option<Vec<Parameter>>, 
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Parameter {
    #[serde(rename = "@ShortName")]
    pub short_name: String,
    #[serde(rename = "@Value")]
    pub value: String,
    #[serde(rename = "@space")]
    pub xml_space: Option<String>, 
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Kommentar {
    #[serde(rename = "@space")]
    pub xml_space: String,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Warten {
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct HexService {

    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "@Phase")]
    pub phase: String,
    #[serde(rename = "@PhaseDetail")]
    pub phase_detail: String,
    #[serde(rename = "@did")]
    pub did: Option<String>,   
    #[serde(rename = "@Bewertung")]
    pub bewertung: String,
    #[serde(rename = "Kommentar")]
    pub kommentar: Kommentar,
    #[serde(rename = "Request")]
    pub request: String,
    #[serde(rename = "ExpectedValue")]
    pub expected_value: Option<String>,
    #[serde(rename = "Response")]
    pub response: Option<String>,
    #[serde(rename = "HumanTranslations")]
    pub human_translations: HumanTranslations,
    pub sections: Vec<Section>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct HumanTranslations {
    #[serde(rename = "@ServiceID")]
    pub service_id: Option<String>,
    #[serde(rename = "@RDIdentifier")]
    pub rd_identifier: Option<String>,
    #[serde(rename = "@ServiceName")]
    pub service_name: Option<String>,
    #[serde(rename = "Translation")]
    pub translations: Option<Vec<Translations>>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Translations {
    #[serde(rename = "@ParameterName")]
    pub parameter_name: String,
    #[serde(rename = "@BytePosition")]
    pub byte_position: String,
    pub lsb: String,
    #[serde(rename = "@BitLength")]
    pub bit_length: String,
    #[serde(rename = "@HexValue")]
    pub hex_value: String,
    #[serde(rename = "$text")]
    pub value: String,
}

/* Here comes the rest  */

#[derive(Serialize, Debug)]
#[serde(deny_unknown_fields, tag = "@OBJECT")]
pub enum Section {
    ECU(ECUSection)
}

impl Section {
    pub fn get_title(&self) -> &String {
        match self {
            Section::ECU(section) => &section.title,
        }
    }
    pub fn get_measurements(&self) -> &Vec<Measurement> {
        match self {
            Section::ECU(section) => &section.measurements,
        }
    }
    #[allow(dead_code)]
    fn get_measurement_by_title(&self, title: &String) -> Option<Measurement> {
        match self {
            Section::ECU(section) => get_measurement_by_title(&section.measurements, title),
        }
    }
}

impl<'de> Deserialize<'de> for Section {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        UntaggedEnumVisitor::new()
            .map(|map| {
                let value: CommonSection = map.deserialize()?;
                match value.object.as_str() {
                    "ECU" => Ok(Section::ECU(ECUSection::from(value))),
                    _ => {
                        Err(serde::de::Error::custom(format!("'{}' not implemented", value.object.as_str())))
                    }
                }
            })
            .deserialize(deserializer)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct CommonSection {
    #[serde(rename = "@OBJECT")]
    pub object: String,
    #[serde(rename = "TITLE")]
    pub title: String,
    #[serde(rename = "MEAS")]
    pub measurements: Vec<Measurement>,
}

#[derive(Serialize, Debug)]
pub struct ECUSection {
    #[serde(rename = "TITLE")]
    pub title: String,
    #[serde(rename = "MEAS")]
    pub measurements: Vec<Measurement>,
}

impl From<CommonSection> for ECUSection {
    fn from(m: CommonSection) -> Self {
        ECUSection {
            title: m.title,
            measurements: m.measurements,
        }
    }
}

fn get_measurement_by_title(measurements: &Vec<Measurement>, title: &String) -> Option<Measurement> {
    for m in measurements {
        if m.get_title() == title {
            return Some(m.clone());
        }
    }
    None
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum Measurement {
    Codierung(MeasurementCoding),
    Identifikation(MeasurementIdentification),
    Fehler(MeasurementMistake),
    Messwerte(MeasurementMeasuredValues),
    ErweiterterFehlerspeicher(MeasurementExtendedErrorMemory),
}

impl Measurement {
    pub fn get_title(&self) -> &String {
        match self {
            Measurement::Codierung(m) => &m.title,
            Measurement::Identifikation(m) => &m.title,
            Measurement::Fehler(m) => &m.title,
            Measurement::Messwerte(m) => &m.title,
            Measurement::ErweiterterFehlerspeicher(m) => &m.title,
        }
    }

    pub fn get_values(&self) -> Option<&Vec<ValueEnum>> {
        match self {
            Measurement::Codierung(m) => m.values.as_ref(),
            Measurement::Identifikation(m) => m.values.as_ref(),
            Measurement::Fehler(m) => m.values.as_ref(),
            Measurement::Messwerte(m) => m.values.as_ref(),
            Measurement::ErweiterterFehlerspeicher(m) => m.values.as_ref(),
        }
    }

    #[allow(dead_code)]
    fn get_value_by_label(&self, label: &String) -> Option<&ValueEnum> {
        match self.get_values() {
            Some(values) => values.iter().find(|v| match v {
                ValueEnum::Num(n) => &n.label == label,
                ValueEnum::Alpha(a) => &a.label == label,
            }),
            _ => None,
        }
    }

    pub fn get_submeasurements(&self) -> Option<&Vec<Measurement>> {
        match self {
            Measurement::Fehler(m) => m.measurements.as_ref(),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn get_submeasurement_by_title(&self, title: &String) -> Option<Measurement> {
        match self {
            Measurement::Fehler(m) => match &m.measurements {
                Some(measurements) => get_measurement_by_title(&measurements, title),
                _ => None,
            }
            _ => None
        }
    }
}

impl<'de> Deserialize<'de> for Measurement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        UntaggedEnumVisitor::new()
            .map(|map| {
                let value: CommonMeasurement = map.deserialize()?;
                match value.object.as_str() {
                    "Codierung" => Ok(Measurement::Codierung(MeasurementCoding::from(value))),
                    "Identifikation" => Ok(Measurement::Identifikation(MeasurementIdentification::from(value))),
                    "Fehler" => Ok(Measurement::Fehler(MeasurementMistake::from(value))),
                    "Messwerte" => Ok(Measurement::Messwerte(MeasurementMeasuredValues::from(value))),
                    "Erweiterter Fehlerspeicher" => Ok(Measurement::ErweiterterFehlerspeicher(MeasurementExtendedErrorMemory::from(value))),
                    _ => {
                        Err(serde::de::Error::custom(format!("'{}' not implemented", value.object.as_str())))
                    }
                }
            })
            .deserialize(deserializer)
    }
}


#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
struct CommonMeasurement {
    #[serde(rename = "@OBJECT")]
    object: String,
    #[serde(rename = "TITLE")]
    title: String,
    #[serde(rename = "VALUE")]
    values: Option<Vec<ValueEnum>>,
    #[serde(rename = "MEAS")]
    measurements: Option<Vec<Measurement>>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MeasurementCoding {
    pub title: String,
    pub values: Option<Vec<ValueEnum>>,
}

impl From<CommonMeasurement> for MeasurementCoding {
    fn from(m: CommonMeasurement) -> Self {
        if m.measurements != None {
            panic!("unexpected measurements for MeasurementCoding");
        }
        MeasurementCoding {
            title: m.title,
            values: m.values,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MeasurementIdentification {
    pub title: String,
    pub values: Option<Vec<ValueEnum>>,
}

impl From<CommonMeasurement> for MeasurementIdentification {
    fn from(m: CommonMeasurement) -> Self {
        if m.measurements != None {
            panic!("unexpected measurements for MeasurementIdentification");
        }
        MeasurementIdentification {
            title: m.title,
            values: m.values,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MeasurementMeasuredValues {
    pub title: String,
    pub values: Option<Vec<ValueEnum>>,
}

impl From<CommonMeasurement> for MeasurementMeasuredValues {
    fn from(m: CommonMeasurement) -> Self {
        if m.measurements != None {
            panic!("unexpected measurements for MeasurementMeasuredValues");
        }
        MeasurementMeasuredValues {
            title: m.title,
            values: m.values,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MeasurementMistake {
    pub title: String,
    pub values: Option<Vec<ValueEnum>>,
    pub measurements: Option<Vec<Measurement>>,
}

impl From<CommonMeasurement> for MeasurementMistake {
    fn from(m: CommonMeasurement) -> Self {
        MeasurementMistake {
            title: m.title,
            values: m.values,
            measurements: m.measurements,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MeasurementExtendedErrorMemory {
    pub title: String,
    pub values: Option<Vec<ValueEnum>>,
}

impl From<CommonMeasurement> for MeasurementExtendedErrorMemory {
    fn from(m: CommonMeasurement) -> Self {
        MeasurementExtendedErrorMemory {
            title: m.title,
            values: m.values,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE", tag = "@FORMAT")]
pub enum ValueEnum {
    Num(NumberValue),
    Alpha(AlphaValue),
}

impl ValueEnum {
    pub fn get_label(&self) -> &String {
        match self {
            ValueEnum::Num(n) => &n.label,
            ValueEnum::Alpha(a) => &a.label,
        }
    }

    pub fn get_text(&self) -> &String {
        match self {
            ValueEnum::Num(n) => &n.text,
            ValueEnum::Alpha(a) => &a.text,
        }
    }

    pub fn get_unit(&self) -> Option<&String> {
        match self {
            ValueEnum::Num(n) => n.unit.as_ref(),
            _ => None,
        }
    }

    pub fn get_value(&self) -> Option<&String> {
        match self {
            ValueEnum::Num(n) => Some(&n.value),
            ValueEnum::Alpha(a) => a.value.as_ref(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct NumberValue {
    #[serde(rename = "@TEXT")]
    pub text: String,
    #[serde(rename = "@UNIT")]
    pub unit: Option<String>,
    #[serde(rename = "@LABEL")]
    pub label: String,
    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AlphaValue {
    #[serde(rename = "@TEXT")]
    pub text: String,
    #[serde(rename = "@LABEL")]
    pub label: String,
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let val = VehicleAnalysisLog::from_zip("tests/data/FAP_XXXXXXXXXXXXXXXXX_20240804_132559_23.0.1.zip").unwrap();
        assert_eq!(val.results_header.vehicle.ident.vin, "XXXXXXXXXXXXXXXXX");
        assert_eq!(val.result.header.equipment.pt2g_version, "42.200.010");
        assert_eq!(val.result.header.timezone, FixedOffset::west_opt(7 * 3600).unwrap());

        let section = &val.get_section_by_title("Gateway (A7.1)").unwrap();
        let m = &section.get_measurement_by_title(&"Control unit, coding".to_string()).unwrap();
        let value = m.get_value_by_label(&"Batteriewechsel_Technologie_zwei.Scannercode".to_string()).unwrap();
        assert_eq!(m.get_title(), "Control unit, coding");
        assert_eq!(value, &ValueEnum::Alpha(AlphaValue {
            text: "Battery change: Scanner code".to_string(),
            label: "Batteriewechsel_Technologie_zwei.Scannercode".to_string(),
            value: Some("205 BA24H9F0EGE".to_string()),
        }));


        let section = &val.get_section_by_title("Airbag (variant: A2.8)").unwrap();
        let m = &section.get_measurement_by_title(&"Fault".to_string()).unwrap();
        let submeasurement = m.get_submeasurement_by_title(&"erweiterter Fehlerspeicher".to_string()).unwrap();
        let value = submeasurement.get_value_by_label(&"Priority".to_string()).unwrap();
        assert_eq!(m.get_title(), "Fault");
        assert_eq!(value, &ValueEnum::Alpha(AlphaValue {
            text: "Hinweis_Prio".to_string(),
            label: "Priority".to_string(),
            value: Some("2".to_string()),
        }));
    }
}
