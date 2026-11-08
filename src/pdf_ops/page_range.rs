pub fn parse_page_ranges(input: &str, max_pages: u32) -> Result<Vec<u32>, String> {
    if max_pages == 0 {
        return Err("Dokument hat keine Seiten".to_string());
    }

    let trimmed = input.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("alle")
        || trimmed.eq_ignore_ascii_case("all")
    {
        return Ok((1..=max_pages).collect());
    }
    if trimmed.eq_ignore_ascii_case("gerade") || trimmed.eq_ignore_ascii_case("even") {
        let pages: Vec<u32> = (1..=max_pages).filter(|p| p % 2 == 0).collect();
        if pages.is_empty() {
            return Err("Keine geraden Seiten im Dokument vorhanden".to_string());
        }
        return Ok(pages);
    }
    if trimmed.eq_ignore_ascii_case("ungerade") || trimmed.eq_ignore_ascii_case("odd") {
        return Ok((1..=max_pages).filter(|p| p % 2 != 0).collect());
    }

    let mut pages = Vec::new();
    for part in trimmed.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if let Some((start_str, end_str)) = part.split_once('-') {
            let start_str = start_str.trim();
            let end_str = end_str.trim();

            let start: u32 = if start_str.is_empty() {
                1
            } else {
                start_str
                    .parse()
                    .map_err(|_| format!("Ungültige Startseite im Bereich: '{part}'"))?
            };

            let end: u32 = if end_str.is_empty() {
                max_pages
            } else {
                end_str
                    .parse()
                    .map_err(|_| format!("Ungültige Endseite im Bereich: '{part}'"))?
            };

            if start == 0 || end == 0 {
                return Err(format!("Seitennummern müssen mindestens 1 sein: '{part}'"));
            }
            if start > end {
                return Err(format!(
                    "Startseite ({start}) darf nicht größer als Endseite ({end}) sein: '{part}'"
                ));
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

/// Parses a page sequence preserving user order, allowing duplicates and reverse ranges (e.g. "3, 1, 5-2, 1").
pub fn parse_page_sequence(input: &str, max_pages: u32) -> Result<Vec<u32>, String> {
    if max_pages == 0 {
        return Err("Dokument hat keine Seiten".to_string());
    }

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Bitte eine Seitenreihenfolge eingeben".to_string());
    }
    if trimmed.eq_ignore_ascii_case("umkehren") || trimmed.eq_ignore_ascii_case("reverse") {
        return Ok((1..=max_pages).rev().collect());
    }

    let mut seq = Vec::new();
    for part in trimmed.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if let Some((start_str, end_str)) = part.split_once('-') {
            let start: u32 = if start_str.trim().is_empty() {
                1
            } else {
                start_str
                    .trim()
                    .parse()
                    .map_err(|_| format!("Ungültige Startseite: '{part}'"))?
            };
            let end: u32 = if end_str.trim().is_empty() {
                max_pages
            } else {
                end_str
                    .trim()
                    .parse()
                    .map_err(|_| format!("Ungültige Endseite: '{part}'"))?
            };

            if start == 0 || end == 0 || start > max_pages || end > max_pages {
                return Err(format!(
                    "Seitenbereich '{part}' liegt außerhalb von 1..={max_pages}"
                ));
            }

            if start <= end {
                seq.extend(start..=end);
            } else {
                seq.extend((end..=start).rev());
            }
        } else {
            let page: u32 = part
                .parse()
                .map_err(|_| format!("Ungültige Seitennummer: '{part}'"))?;
            if page == 0 || page > max_pages {
                return Err(format!("Seite {page} liegt außerhalb von 1..={max_pages}"));
            }
            seq.push(page);
        }
    }

    if seq.is_empty() {
        return Err("Keine gültigen Seiten in der Sequenz".to_string());
    }
    Ok(seq)
}
