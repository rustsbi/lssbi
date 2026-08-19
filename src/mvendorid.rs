// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use jep106::JEP106Code;

pub(crate) fn vendor_name(id: usize) -> Option<&'static str> {
    let bank = u8::try_from(id >> 7).ok()?;
    JEP106Code::new(bank, (id & 0x7f) as u8).get()
}

#[cfg(test)]
mod tests {
    use super::vendor_name;

    #[test]
    fn decodes_jep106_id() {
        assert_eq!(vendor_name(0x144), Some("Nordic VLSI ASA"));
    }

    #[test]
    fn rejects_unknown_or_oversized_ids() {
        assert_eq!(vendor_name(0), None);
        assert_eq!(vendor_name(usize::MAX), None);
    }
}
