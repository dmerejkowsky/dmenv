use spectral::Spec;

pub trait IsAbsentFrom {
    fn is_absent_from(&mut self, text: &str);
}

impl<'s> IsAbsentFrom for Spec<'s, &str> {
    fn is_absent_from(&mut self, text: &str) {
        let subject = self.subject;
        if text.contains(subject) {
            panic!("\n{}\nshould be absent from\n{}", subject, text);
        }
    }
}

pub trait IsPresentIn {
    fn is_present_in(&mut self, text: &str);
}

impl<'s> IsPresentIn for Spec<'s, &str> {
    fn is_present_in(&mut self, text: &str) {
        let subject = self.subject;
        if !text.contains(subject) {
            panic!("\n{}\nshould be present in\n{}", subject, text);
        }
    }
}
