pub fn parse_page_ranges(input: &str, max_pages: u32) -> Result<Vec<u32>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("alle") || trimmed.eq_ignore_ascii_case("all") {
        return Ok((1..=max_pages).collect());
    }

    let mut pages = Vec::new();
    let parts = trimmed.split(',');

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if let Some((start_str, end_str)) = part.split_once('-') {
            let start: u32 = start_str
                .trim()
                .parse()
                .map_err(|_| format!("Ungültige Startseite im Bereich: '{part}'"))?;
            let end: u32 = end_str
                .trim()
                .parse()
                .map_err(|_| format!("Ungültige Endseite im Bereich: '{part}'"))?;

            if start == 0 || end == 0 {
                return Err(format!("Seitennummern müssen mindestens 1 sein: '{part}'"));
            }
            if start > end {
                return Err(format!("Startseite ({start}) darf nicht größer als Endseite ({end}) sein: '{part}'"));
            }
            if start > max_pages {
                return Err(format!("Seite {start} überschreitet Gesamtzahl ({max_pages})"));
            }
            let clamped_end = end.min(max_pages);
            for p in start..=clamped_end {
                if !pages.contains(&p) {
                    pages.push(p);
                }
            }
        } else {
            let page: u32 = part
                .parse()
                .map_err(|_| format!("Ungültige Seitennummer: '{part}'"))?;
            if page == 0 {
                return Err("Seitennummer darf nicht 0 sein".to_string());
            }
            if page > max_pages {
                return Err(format!("Seite {page} überschreitet Dokumentseiten ({max_pages})"));
            }
            if !pages.contains(&page) {
                pages.push(page);
            }
        }
    }

    pages.sort_unstable();
    if pages.is_empty() {
        return Err("Keine gültigen Seiten ausgewählt".to_string());
    }
    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ranges() {
        assert_eq!(parse_page_ranges("1-3, 5, 7-8", 10).unwrap(), vec![1, 2, 3, 5, 7, 8]);
        assert_eq!(parse_page_ranges("alle", 5).unwrap(), vec![1, 2, 3, 4, 5]);
        assert_eq!(parse_page_ranges("", 3).unwrap(), vec![1, 2, 3]);
    }
}
