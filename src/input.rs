use std::path::Path;

use crate::model::{Model, ModelMessages, ModelMonetary, ModelNumeric, ModelTime};
use anyhow::{Context, Result, anyhow};
use roxmltree::{Document, ParsingOptions};
use tokio::fs::read_to_string;

pub async fn get_locale_data(
    language: &str,
    region: &str,
    messages: ModelMessages,
) -> Result<Model> {
    let tree_set_str = TreeSetStr::get(Path::new("./cldr/common"), language, region).await?;
    from_xml(region, &tree_set_str, messages)
}

struct TreeSetStr {
    suplemental_data: String,
    main_language: String,
    main_region: String,
    main_root: String,
}

impl TreeSetStr {
    async fn get(cldr_common: &Path, language: &str, region: &str) -> Result<Self> {
        Ok(TreeSetStr {
            suplemental_data: read_to_string(cldr_common.join("supplemental/supplementalData.xml"))
                .await
                .context("while reading supplemental")?,
            main_language: read_to_string(&cldr_common.join(&format!("main/{language}.xml")))
                .await?,
            // Not every locale has a file for its region, when it doesn't exist the
            // data is inherited from the language file.
            main_region: get_optional_text(
                &cldr_common.join(&format!("main/{language}_{region}.xml")),
            )
            .await,
            main_root: read_to_string(cldr_common.join("main/root.xml")).await?,
        })
    }

    fn parse<'a>(&'a self) -> Result<TreeSetDoc<'a>> {
        fn parse_xml<'a>(xml: &'a str) -> Result<Document<'a>> {
            Ok(Document::parse_with_options(
                xml,
                ParsingOptions {
                    allow_dtd: true,
                    ..Default::default()
                },
            )?)
        }

        Ok(TreeSetDoc {
            suplemental_data: parse_xml(&self.suplemental_data)
                .context("While parsing suplemental data")?,
            main_region: parse_xml(&self.main_region).context("While parsing main region")?,
            main_language: parse_xml(&self.main_language).context("Wihle parsing main language")?,
            main_root: parse_xml(&self.main_root).context("While parsing main root")?,
        })
    }
}

struct TreeSetDoc<'a> {
    suplemental_data: Document<'a>,
    main_language: Document<'a>,
    main_region: Document<'a>,
    main_root: Document<'a>,
}

impl<'a> TreeSetDoc<'a> {
    fn get_roots(&'a self, region: &'a str) -> TreeSet<'a> {
        TreeSet {
            region,
            suplemental_data: XmlNode(self.suplemental_data.root_element()),
            main_language: XmlNode(self.main_language.root_element()),
            main_region: XmlNode(self.main_region.root_element()),
            main_root: XmlNode(self.main_root.root_element()),
        }
    }
}

struct TreeSet<'a> {
    region: &'a str,
    suplemental_data: XmlNode<'a, 'a>,
    main_language: XmlNode<'a, 'a>,
    main_region: XmlNode<'a, 'a>,
    main_root: XmlNode<'a, 'a>,
}

impl<'a> TreeSet<'a> {
    fn get_currency_id(&self) -> Result<&str> {
        self.suplemental_data
            .select(&format!(
                "currencyData > region[iso3166={}] > currency:first-child",
                self.region
            ))
            .ok_or(anyhow!("Failed to find currency node"))?
            .attribute("iso4217")
            .ok_or(anyhow!("Couldn't find currency name"))
    }

    /// The XML documents where the data is looked for, from the most specific one
    /// (the region) to the least specific one (the root locale).
    fn documents(&self) -> Vec<XmlNode<'a, 'a>> {
        vec![self.main_region, self.main_language, self.main_root]
    }

    /// The numbering systems to look for, the default one of the locale comes first.
    fn numbering_systems(&self) -> Vec<String> {
        let mut systems = Vec::new();

        for document in self.documents() {
            let Some(numbering_system) = document
                .select("numbers > defaultNumberingSystem")
                .and_then(|node| node.text())
            else {
                continue;
            };

            // The value may be inherited from the parent locale.
            if numbering_system == INHERITED {
                continue;
            }

            if !systems.iter().any(|system| system == numbering_system) {
                systems.push(numbering_system.to_string());
            }

            break;
        }

        if !systems.iter().any(|system| system == "latn") {
            systems.push("latn".to_string());
        }

        systems
    }

    /// The default numbering system of the locale.
    fn numbering_system(&self) -> String {
        self.numbering_systems()
            .into_iter()
            .next()
            .unwrap_or_else(|| "latn".to_string())
    }

    /// The text of the first node matching the selector that isn't inherited from the
    /// parent locale, following the CLDR aliases.
    fn find(&self, selector: &str) -> Option<&'a str> {
        let mut path: Vec<String> = selector
            .split('>')
            .map(str::trim)
            .map(str::to_string)
            .collect();

        // The aliases could create a loop, so the number of redirects is bounded.
        for _ in 0..32 {
            let selector = path.join(" > ");
            let mut redirect = None;

            for document in self.documents() {
                let value = document
                    .select(&selector)
                    .map(|node| node.text().unwrap_or(""))
                    .filter(|text| *text != INHERITED);

                if let Some(value) = value {
                    return Some(value);
                }

                // The selector didn't match, look for an alias in its place.
                match lookup_path(&document, &path) {
                    Lookup::Value(value) if value == INHERITED => continue,
                    Lookup::Value(value) => return Some(value),
                    Lookup::Alias(alias) => {
                        redirect = Some(alias);
                        break;
                    }
                    Lookup::Missing => continue,
                }
            }

            path = redirect?;
        }

        None
    }

    /// The currency pattern of the locale, `alt` selects the variant of the pattern
    /// (`None` is the default one and `Some("alphaNextToNumber")` the one used when the
    /// currency symbol is written with letters, as it happens with the international
    /// currency symbol).
    fn currency_pattern(&self, alt: Option<&str>) -> Option<String> {
        let numbering_systems = self.numbering_systems();

        // The numbering systems are checked before the documents because a locale inherits
        // the data of its numbering system from the parent locale, and not the data of
        // another numbering system.
        for numbering_system in &numbering_systems {
            for document in self.documents() {
                if let Some(pattern) = select_currency_pattern(&document, numbering_system, alt) {
                    return Some(pattern);
                }
            }
        }

        None
    }
}

/// An empty locale document, used when CLDR has no file for the locale.
const EMPTY_LOCALE: &str = r#"<?xml version="1.0" encoding="UTF-8"?><ldml/>"#;

/// The value CLDR uses to mark a value that is inherited from the parent locale.
const INHERITED: &str = "\u{2191}\u{2191}\u{2191}";

/// The result of looking for a path in one of the locale documents.
enum Lookup<'a> {
    Value(&'a str),
    Alias(Vec<String>),
    Missing,
}

/// Applies an alias path, the `..` components move to the parent element.
fn apply_alias(path: &mut Vec<String>, alias: &str) {
    for component in alias.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                path.pop();
            }
            // The alias paths use `[@attribute]`, the selectors use `[attribute]`.
            component => path.push(component.replace("[@", "[")),
        }
    }
}

/// Looks for a path of child elements in a document using CSS selectors. It walks the
/// path one element at a time so it can detect an alias in place of a missing child.
fn lookup_path<'a>(document: &XmlNode<'a, 'a>, path: &[String]) -> Lookup<'a> {
    let mut current = *document;

    for (index, component) in path.iter().enumerate() {
        let Some(child) = current.select_child(component) else {
            let Some(alias) = current
                .select_child("alias")
                .and_then(|node| node.attribute("path"))
            else {
                return Lookup::Missing;
            };

            let mut alias_path: Vec<String> = path[..index].to_vec();
            apply_alias(&mut alias_path, alias);
            alias_path.extend_from_slice(&path[index..]);

            return Lookup::Alias(alias_path);
        };

        current = XmlNode(child);
    }

    Lookup::Value(current.0.text().unwrap_or(""))
}

/// Looks for the currency pattern of a numbering system inside a document.
fn select_currency_pattern(
    document: &XmlNode<'_, '_>,
    numbering_system: &str,
    alt: Option<&str>,
) -> Option<String> {
    let format_length = document
        .select_all(&format!(
            "numbers > currencyFormats[numberSystem='{numbering_system}'] > currencyFormatLength"
        ))
        .into_iter()
        .find(|node| node.0.attribute("type").is_none())?;

    let currency_format = format_length.select("currencyFormat[type='standard']")?;

    XmlNode(currency_format)
        .select_all("pattern")
        .into_iter()
        .find(|node| node.0.attribute("alt") == alt)
        .and_then(|node| node.0.text())
        .filter(|text| *text != INHERITED)
        .map(str::to_string)
}

#[derive(Clone, Copy)]
struct XmlNode<'a, 'input: 'a>(roxmltree::Node<'a, 'input>);

impl<'a, 'input: 'a> XmlNode<'a, 'input> {
    fn select(&self, text: &str) -> Option<roxmltree::Node<'a, 'input>> {
        let selectors = simplecss::Selector::parse(text)?;
        self.0
            .descendants()
            .filter(|n| n.is_element())
            .find(|&node| selectors.matches(&XmlNode(node)))
    }

    fn select_all(&self, text: &str) -> Vec<XmlNode<'a, 'input>> {
        let Some(selectors) = simplecss::Selector::parse(text) else {
            return Vec::new();
        };

        self.0
            .descendants()
            .filter(|n| n.is_element())
            .filter(|&node| selectors.matches(&XmlNode(node)))
            .map(XmlNode)
            .collect()
    }

    fn select_child(&self, text: &str) -> Option<roxmltree::Node<'a, 'input>> {
        let selectors = simplecss::Selector::parse(text)?;
        // Without an attribute filter, CLDR means the default element, the one without
        // `type` or `alt` attributes.
        let has_filter = text.contains('[');

        self.0.children().filter(|n| n.is_element()).find(|&node| {
            selectors.matches(&XmlNode(node))
                && (has_filter
                    || (node.attribute("type").is_none() && node.attribute("alt").is_none()))
        })
    }
}

impl simplecss::Element for XmlNode<'_, '_> {
    fn parent_element(&self) -> Option<Self> {
        self.0.parent_element().map(XmlNode)
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.0.prev_sibling_element().map(XmlNode)
    }

    fn has_local_name(&self, local_name: &str) -> bool {
        self.0.tag_name().name() == local_name
    }

    fn attribute_matches(
        &self,
        local_name: &str,
        operator: simplecss::AttributeOperator<'_>,
    ) -> bool {
        matches!(self.0.attribute(local_name), Some(value) if operator.matches(value))
    }

    fn pseudo_class_matches(&self, class: simplecss::PseudoClass<'_>) -> bool {
        match class {
            simplecss::PseudoClass::FirstChild => self.prev_sibling_element().is_none(),
            _ => false, // Since we are querying a static XML we can ignore other pseudo-classes.
        }
    }
}

fn from_xml(region: &str, set: &TreeSetStr, messages: ModelMessages) -> Result<Model> {
    let tree_set_doc = set.parse()?;
    let tree_set = tree_set_doc.get_roots(region);

    Ok(Model {
        time: extract_time(&tree_set)?,
        numeric: extract_numeric(&tree_set)?,
        monetary: extract_monetary(&tree_set)?,
        messages,
    })
}

/// Maps a CLDR date field to its `strftime` equivalent, the width is the number of
/// times the field is repeated in the pattern.
fn strftime_field(field: char, width: usize) -> Option<&'static str> {
    Some(match field {
        // The eras are handled by the `era_*` formats.
        'G' => "",
        'y' => match width {
            2 => "%y",
            _ => "%Y",
        },
        'Y' => match width {
            2 => "%g",
            _ => "%G",
        },
        'u' | 'U' | 'r' => "%Y",
        'M' | 'L' => match width {
            1 | 2 => "%m",
            3 => "%b",
            _ => "%B",
        },
        'd' => "%d",
        'D' => "%j",
        'E' => {
            if width >= 4 {
                "%A"
            } else {
                "%a"
            }
        }
        'c' => match width {
            1 | 2 => "%u",
            3 | 5 | 6 => "%a",
            _ => "%A",
        },
        'e' => "%u",
        'a' | 'b' | 'B' => "%p",
        'h' | 'K' => "%I",
        'H' | 'k' | 'j' => "%H",
        'm' => "%M",
        's' => "%S",
        'w' => "%U",
        'z' | 'v' | 'V' | 'O' => "%Z",
        'Z' | 'X' | 'x' => "%z",
        // Fields without an equivalent in `strftime`.
        'Q' | 'q' | 'W' | 'F' | 'g' | 'S' | 'A' | 'J' | 'C' => "",
        _ => return None,
    })
}

/// Converts a CLDR date pattern into a `strftime` pattern.
fn to_strftime(pattern: &str) -> String {
    to_formatted_strftime(pattern, &[])
}

/// Converts a CLDR pattern into a `strftime` pattern, the `{0}` and `{1}` placeholders
/// used by the date time patterns are replaced with the given arguments.
fn to_formatted_strftime(pattern: &str, arguments: &[(&str, &str)]) -> String {
    let mut result = String::new();
    let mut characters = pattern.chars().peekable();
    let mut literal = false;

    while let Some(character) = characters.next() {
        if character == '\'' {
            if literal && characters.peek() == Some(&'\'') {
                characters.next();
                result.push('\'');
            } else {
                literal = !literal;
            }
            continue;
        }

        if literal {
            result.push(character);
            continue;
        }

        if character == '{' {
            let mut placeholder = String::from(character);
            for next in characters.by_ref() {
                placeholder.push(next);
                if next == '}' {
                    break;
                }
            }

            match arguments.iter().find(|(name, _)| *name == placeholder) {
                Some((_, argument)) => result.push_str(argument),
                None => result.push_str(&placeholder),
            }
            continue;
        }

        if character.is_ascii_alphabetic() {
            let mut width = 1;
            while characters.peek() == Some(&character) {
                characters.next();
                width += 1;
            }

            if let Some(field) = strftime_field(character, width) {
                result.push_str(field);
            }
            continue;
        }

        // The bidi control characters are not part of the formatted output.
        if matches!(character, '\u{200e}' | '\u{200f}' | '\u{061c}') {
            continue;
        }

        result.push(character);
    }

    result
}

/// The symbol of the default numbering system of the locale, an empty symbol is valid
/// (some locales don't use a thousands separator for example).
fn symbol(tree_set: &TreeSet<'_>, name: &str) -> Option<String> {
    let number_system = tree_set.numbering_system();

    tree_set
        .find(&format!(
            "numbers > symbols[numberSystem='{number_system}'] > {name}"
        ))
        .map(str::to_string)
}

/// The decimal pattern of the default numbering system of the locale.
fn decimal_pattern<'a>(tree_set: &TreeSet<'a>) -> Option<&'a str> {
    let number_system = tree_set.numbering_system();

    for document in tree_set.documents() {
        let Some(format_length) = document
            .select_all(&format!(
                "numbers > decimalFormats[numberSystem='{number_system}'] > decimalFormatLength"
            ))
            .into_iter()
            .find(|node| node.0.attribute("type").is_none())
        else {
            continue;
        };

        let Some(pattern) = format_length
            .select_all("decimalFormat > pattern")
            .into_iter()
            .find(|node| node.0.attribute("type").is_none() && node.0.attribute("alt").is_none())
        else {
            continue;
        };

        if let Some(text) = pattern.0.text().filter(|text| *text != INHERITED) {
            return Some(text);
        }
    }

    None
}

/// The size of the groups of the integer part of a number pattern.
fn grouping_size(pattern: &str) -> i8 {
    let pattern = pattern.split(';').next().unwrap_or(pattern);
    let integer_part = pattern.split('.').next().unwrap_or(pattern);

    let Some((_, last_group)) = integer_part.rsplit_once(',') else {
        return -1;
    };

    last_group
        .chars()
        .filter(|character| matches!(character, '0' | '#'))
        .count() as i8
}

fn extract_time(tree_set: &TreeSet<'_>) -> Result<ModelTime> {
    const CALENDAR: &str = "dates > calendars > calendar[type='gregorian']";
    const DAYS: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
    const MONTHS: [&str; 12] = [
        "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12",
    ];

    fn list<const N: usize>(
        tree_set: &TreeSet<'_>,
        mut selector: impl FnMut(&str) -> String,
        types: [&str; N],
    ) -> Result<[String; N]> {
        let mut values = Vec::with_capacity(N);

        for value_type in types {
            let value = tree_set
                .find(&selector(value_type))
                .ok_or_else(|| anyhow!("Failed to find the localized value '{value_type}'"))?;

            values.push(value.to_string());
        }

        values
            .try_into()
            .map_err(|_| anyhow!("Failed to build the localized list"))
    }

    fn days(tree_set: &TreeSet<'_>, width: &str) -> Result<[String; 7]> {
        list(
            tree_set,
            |day| {
                format!(
                    "{CALENDAR} > days > dayContext[type='format'] > dayWidth[type='{width}'] > day[type='{day}']"
                )
            },
            DAYS,
        )
    }

    fn months(tree_set: &TreeSet<'_>, width: &str) -> Result<[String; 12]> {
        list(
            tree_set,
            |month| {
                format!(
                    "{CALENDAR} > months > monthContext[type='format'] > monthWidth[type='{width}'] > month[type='{month}']"
                )
            },
            MONTHS,
        )
    }

    fn period(tree_set: &TreeSet<'_>, period: &str) -> Result<String> {
        for width in ["wide", "abbreviated"] {
            let selector = format!(
                "{CALENDAR} > dayPeriods > dayPeriodContext[type='format'] > dayPeriodWidth[type='{width}'] > dayPeriod[type='{period}']"
            );

            if let Some(value) = tree_set.find(&selector) {
                return Ok(value.to_string());
            }
        }

        Err(anyhow!("Failed to find the '{period}' day period"))
    }

    fn am_pm(tree_set: &TreeSet<'_>) -> Result<[String; 2]> {
        Ok([period(tree_set, "am")?, period(tree_set, "pm")?])
    }

    fn format_pattern<'a>(
        tree_set: &TreeSet<'a>,
        container: &str,
        length: &str,
    ) -> Option<&'a str> {
        let element = container.strip_suffix('s').unwrap_or(container);

        tree_set.find(&format!(
            "{CALENDAR} > {container} > {element}Length[type='{length}'] > {element} > pattern"
        ))
    }

    fn date_pattern(tree_set: &TreeSet<'_>, length: &str) -> Result<String> {
        format_pattern(tree_set, "dateFormats", length)
            .map(to_strftime)
            .ok_or_else(|| anyhow!("Failed to find the '{length}' date pattern"))
    }

    fn time_pattern<'a>(tree_set: &TreeSet<'a>, length: &str) -> Result<&'a str> {
        format_pattern(tree_set, "timeFormats", length)
            .ok_or_else(|| anyhow!("Failed to find the '{length}' time pattern"))
    }

    fn d_fmt(tree_set: &TreeSet<'_>) -> Result<String> {
        date_pattern(tree_set, "short")
    }

    fn t_fmt(tree_set: &TreeSet<'_>) -> Result<String> {
        Ok(to_strftime(time_pattern(tree_set, "medium")?))
    }

    fn t_fmt_ampm(tree_set: &TreeSet<'_>) -> Result<String> {
        let pattern = time_pattern(tree_set, "medium")?;

        // Only the locales that use a 12-hour clock have a representation with AM/PM,
        // the rest use the default one.
        if pattern.contains(['h', 'K']) {
            Ok(to_strftime(pattern))
        } else {
            Ok("%I:%M:%S %p".to_string())
        }
    }

    fn d_t_fmt(tree_set: &TreeSet<'_>) -> Result<String> {
        let date = date_pattern(tree_set, "full")?;
        let time = to_strftime(time_pattern(tree_set, "medium")?);

        let pattern = format_pattern(tree_set, "dateTimeFormats", "medium")
            .ok_or_else(|| anyhow!("Failed to find the 'medium' date time pattern"))?;

        Ok(to_formatted_strftime(
            pattern,
            &[("{0}", &time), ("{1}", &date)],
        ))
    }

    Ok(ModelTime {
        abday: days(tree_set, "abbreviated")?,
        day: days(tree_set, "wide")?,
        abmon: months(tree_set, "abbreviated")?,
        mon: months(tree_set, "wide")?,
        am_pm: am_pm(tree_set)?,
        d_t_fmt: d_t_fmt(tree_set)?,
        d_fmt: d_fmt(tree_set)?,
        t_fmt: t_fmt(tree_set)?,
        t_fmt_ampm: t_fmt_ampm(tree_set)?,
        // CLDR doesn't provide the era rules (start and end dates) used by the libc
        // `era` keyword.
        era: String::new(),
        era_d_t_fmt: d_t_fmt(tree_set)?,
        era_d_fmt: d_fmt(tree_set)?,
        era_t_fmt: t_fmt(tree_set)?,
        // CLDR doesn't provide an equivalent for the alternative date digits.
        alt_digits: "0123456789".to_string(),
    })
}

fn extract_numeric(tree_set: &TreeSet<'_>) -> Result<ModelNumeric> {
    let decimal_point =
        symbol(tree_set, "decimal").ok_or_else(|| anyhow!("Failed to find the decimal point"))?;

    let thousands_sep = symbol(tree_set, "group").unwrap_or_default();

    let grouping = decimal_pattern(tree_set).map(grouping_size).unwrap_or(-1);

    Ok(ModelNumeric {
        decimal_point,
        thousands_sep,
        grouping,
    })
}

/// A currency pattern as defined by CLDR, it may have a negative subpattern
/// separated by a `;`, if it doesn't the negative amounts are the positive ones
/// prefixed by a minus sign.
struct CurrencyPattern {
    positive: PatternPart,
    negative: PatternPart,
}

impl CurrencyPattern {
    fn new(pattern: &str) -> Self {
        match pattern.split_once(';') {
            Some((positive, negative)) => Self {
                positive: PatternPart::new(positive),
                negative: PatternPart::new(negative),
            },
            None => Self {
                positive: PatternPart::new(pattern),
                negative: PatternPart::new(&format!("-{pattern}")),
            },
        }
    }

    /// The position of the sign of a positive amount, the positive subpattern has no
    /// sign so the sign is placed where the negative one would be.
    fn positive_sign_posn(&self) -> i8 {
        self.negative.sign_posn(false).unwrap_or(1)
    }

    /// The position of the sign of a negative amount.
    fn negative_sign_posn(&self) -> i8 {
        self.negative.sign_posn(true).unwrap_or(1)
    }
}

/// The patterns used by the national and the international monetary formats.
struct MonetaryPatterns {
    national: CurrencyPattern,
    international: CurrencyPattern,
}

/// One of the subpatterns of a currency pattern.
struct PatternPart {
    text: String,
    /// The position of the currency symbol.
    symbol: Option<usize>,
    /// The position of the sign.
    sign: Option<usize>,
    /// The position of the number.
    number: Option<usize>,
    /// Whether the amount is surrounded by parentheses.
    parenthesized: bool,
}

impl PatternPart {
    fn new(text: &str) -> Self {
        // The old style patterns use the placeholders {0} for the currency symbol and
        // {1} for the number.
        let symbol = text.find("{0}").or_else(|| text.find('\u{00a4}'));
        let number = text.find("{1}").or_else(|| text.find(['#', '0']));
        let sign = text.char_indices().find_map(|(position, character)| {
            matches!(character, '-' | '\u{2212}').then_some(position)
        });
        let parenthesized = text.contains(['(', ')']);

        Self {
            text: text.to_string(),
            symbol,
            sign,
            number,
            parenthesized,
        }
    }

    /// Whether the currency symbol goes before the number.
    fn cs_precedes(&self) -> i8 {
        match (self.symbol, self.number) {
            (Some(symbol), Some(number)) if symbol < number => 1,
            _ => 0,
        }
    }

    /// How the currency symbol, the sign and the number are separated, following the
    /// values defined by POSIX for the `*_sep_by_space` keywords:
    /// - 0: no space separates the currency symbol and the value.
    /// - 1: the space separates the currency symbol from the value.
    /// - 2: the space separates the currency symbol from the sign.
    fn sep_by_space(&self) -> i8 {
        let (Some(symbol), Some(number)) = (self.symbol, self.number) else {
            return 0;
        };

        let (start, end) = if symbol < number {
            (symbol, number)
        } else {
            (number, symbol)
        };

        if !self.text[start..end].chars().any(char::is_whitespace) {
            return 0;
        }

        // There is a space between the currency symbol and the number, the sign tells us
        // whether the space separates the symbol from the sign (2) or the symbol and the
        // sign from the number (1).
        if let Some(sign) = self.sign {
            let (start, end) = if symbol < sign {
                (symbol, sign)
            } else {
                (sign, symbol)
            };

            let number_in_between = self
                .number
                .is_some_and(|number| number > start && number < end);

            if !number_in_between && self.text[start..end].chars().any(char::is_whitespace) {
                return 2;
            }
        }

        1
    }

    /// The position of the sign, following the values defined by POSIX for the
    /// `*_sign_posn` keywords:
    /// - 0: the amount and the currency symbol are surrounded by parentheses.
    /// - 1: the sign precedes the quantity and the currency symbol.
    /// - 2: the sign succeeds the quantity and the currency symbol.
    /// - 3: the sign precedes the currency symbol.
    /// - 4: the sign succeeds the currency symbol.
    fn sign_posn(&self, allow_parentheses: bool) -> Option<i8> {
        if allow_parentheses && self.parenthesized {
            return Some(0);
        }

        let sign = self.sign?;
        let symbol = self.symbol?;
        let number = self.number?;

        if sign < symbol && sign < number {
            Some(1)
        } else if sign > symbol && sign > number {
            Some(2)
        } else if sign > symbol && sign < number {
            Some(4)
        } else {
            // The sign is between the number and the currency symbol.
            Some(3)
        }
    }
}

fn extract_monetary(tree_set: &TreeSet<'_>) -> Result<ModelMonetary> {
    fn int_curr_symbol(tree_set: &TreeSet<'_>) -> Result<String> {
        let currency_id = tree_set.get_currency_id()?;

        assert!(currency_id.len() == 3);

        // The international currency symbol is always followed by a space, it's only
        // printed when the symbol has to be separated from the amount.
        Ok(format!("{currency_id} "))
    }

    fn currency_symbol(tree_set: &TreeSet<'_>) -> Result<String> {
        let currency_id = tree_set.get_currency_id()?;

        // CLDR uses the currency code when the locale has no explicit symbol.
        Ok(tree_set
            .find(&format!(
                "numbers > currencies > currency[type='{currency_id}'] > symbol"
            ))
            .unwrap_or(currency_id)
            .to_string())
    }

    fn mon_decimal_point(tree_set: &TreeSet<'_>) -> Result<String> {
        symbol(tree_set, "decimal").ok_or_else(|| anyhow!("Failed to find the decimal point"))
    }

    fn mon_thousands_sep(tree_set: &TreeSet<'_>) -> Result<String> {
        Ok(symbol(tree_set, "group").unwrap_or_default())
    }

    fn positive_sign(tree_set: &TreeSet<'_>) -> Result<String> {
        Ok(symbol(tree_set, "plusSign").unwrap_or_default())
    }

    fn negative_sign(tree_set: &TreeSet<'_>) -> Result<String> {
        Ok(symbol(tree_set, "minusSign").unwrap_or_default())
    }

    fn int_frac_digits(tree_set: &TreeSet<'_>, currency_code: &str) -> Result<i8> {
        fn extract_digits<'a, 'input>(n: roxmltree::Node<'a, 'input>) -> Result<i8> {
            Ok(n.attribute("digits").ok_or(anyhow!(""))?.parse()?)
        }

        tree_set
            .suplemental_data
            .select(&format!(
                "supplementalData > currencyData > fractions > info[iso4217='{currency_code}']"
            ))
            .map(extract_digits)
            .unwrap_or(Ok(2))
    }

    /// The currency patterns used to build the monetary data, the national one uses the
    /// currency symbol of the locale and the international one the pattern used when the
    /// symbol is written with letters, as it happens with the international currency
    /// symbol (for example `USD`).
    fn currency_patterns(tree_set: &TreeSet<'_>) -> Result<MonetaryPatterns> {
        let national = tree_set
            .currency_pattern(None)
            .ok_or(anyhow!("Failed to find the currency pattern"))?;

        let international = tree_set
            .currency_pattern(Some("alphaNextToNumber"))
            .unwrap_or_else(|| national.clone());

        Ok(MonetaryPatterns {
            national: CurrencyPattern::new(&national),
            international: CurrencyPattern::new(&international),
        })
    }

    fn p_cs_precedes(tree_set: &TreeSet<'_>) -> Result<i8> {
        Ok(currency_patterns(tree_set)?.national.positive.cs_precedes())
    }

    fn p_sep_by_space(tree_set: &TreeSet<'_>) -> Result<i8> {
        Ok(currency_patterns(tree_set)?
            .national
            .positive
            .sep_by_space())
    }

    fn n_cs_precedes(tree_set: &TreeSet<'_>) -> Result<i8> {
        Ok(currency_patterns(tree_set)?.national.negative.cs_precedes())
    }

    fn n_sep_by_space(tree_set: &TreeSet<'_>) -> Result<i8> {
        Ok(currency_patterns(tree_set)?
            .national
            .negative
            .sep_by_space())
    }

    fn p_sign_posn(tree_set: &TreeSet<'_>) -> Result<i8> {
        Ok(currency_patterns(tree_set)?.national.positive_sign_posn())
    }

    fn n_sign_posn(tree_set: &TreeSet<'_>) -> Result<i8> {
        Ok(currency_patterns(tree_set)?.national.negative_sign_posn())
    }

    fn int_p_cs_precedes(tree_set: &TreeSet<'_>) -> Result<i8> {
        Ok(currency_patterns(tree_set)?
            .international
            .positive
            .cs_precedes())
    }

    fn int_p_sep_by_space(tree_set: &TreeSet<'_>) -> Result<i8> {
        Ok(currency_patterns(tree_set)?
            .international
            .positive
            .sep_by_space())
    }

    fn int_n_cs_precedes(tree_set: &TreeSet<'_>) -> Result<i8> {
        Ok(currency_patterns(tree_set)?
            .international
            .negative
            .cs_precedes())
    }

    fn int_n_sep_by_space(tree_set: &TreeSet<'_>) -> Result<i8> {
        Ok(currency_patterns(tree_set)?
            .international
            .negative
            .sep_by_space())
    }

    fn int_p_sign_posn(tree_set: &TreeSet<'_>) -> Result<i8> {
        Ok(currency_patterns(tree_set)?
            .international
            .positive_sign_posn())
    }

    fn int_n_sign_posn(tree_set: &TreeSet<'_>) -> Result<i8> {
        Ok(currency_patterns(tree_set)?
            .international
            .negative_sign_posn())
    }

    let currency_code = tree_set.get_currency_id()?;
    let decimal_point = mon_decimal_point(tree_set)?;
    let thousands_sep = mon_thousands_sep(tree_set)?;
    let pattern = tree_set
        .currency_pattern(None)
        .ok_or(anyhow!("Failed to get the currency pattern"))?;

    Ok(ModelMonetary {
        int_curr_symbol: int_curr_symbol(tree_set)?,
        currency_symbol: currency_symbol(tree_set)?,
        mon_grouping: grouping_size(&pattern),
        int_frac_digits: int_frac_digits(tree_set, currency_code)?,
        frac_digits: int_frac_digits(tree_set, currency_code)?,
        mon_decimal_point: decimal_point,
        mon_thousands_sep: thousands_sep,
        positive_sign: positive_sign(tree_set)?,
        negative_sign: negative_sign(tree_set)?,
        p_cs_precedes: p_cs_precedes(tree_set)?,
        p_sep_by_space: p_sep_by_space(tree_set)?,
        n_cs_precedes: n_cs_precedes(tree_set)?,
        n_sep_by_space: n_sep_by_space(tree_set)?,
        p_sign_posn: p_sign_posn(tree_set)?,
        n_sign_posn: n_sign_posn(tree_set)?,
        int_p_cs_precedes: int_p_cs_precedes(tree_set)?,
        int_p_sep_by_space: int_p_sep_by_space(tree_set)?,
        int_n_cs_precedes: int_n_cs_precedes(tree_set)?,
        int_n_sep_by_space: int_n_sep_by_space(tree_set)?,
        int_p_sign_posn: int_p_sign_posn(tree_set)?,
        int_n_sign_posn: int_n_sign_posn(tree_set)?,
    })
}

/// Fetches the URL, a missing document is returned as an empty locale so its data is
/// inherited from the other documents.
async fn get_optional_text(path: &Path) -> String {
    tokio::fs::read_to_string(path)
        .await
        .unwrap_or_else(|_| EMPTY_LOCALE.to_string())
}
