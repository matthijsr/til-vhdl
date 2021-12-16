use tydi_common::traits::Document;

pub trait VhdlDocument {
    fn vhdl_doc(&self) -> Option<String>;
}

impl<T: Document> VhdlDocument for T {
    fn vhdl_doc(&self) -> Option<String> {
        if let Some(doc) = self.doc() {
            let mut result = String::new();
            for line in doc.split_terminator('\n') {
                result.push_str(format!("-- {}\n", line).as_str());
            }
            Some(result)
        } else {
            None
        }
    }
}
