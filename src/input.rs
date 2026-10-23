use crate::model::{Model, ModelMessages, ModelMonetary, ModelNumeric, ModelTime};
use anyhow::{Result, anyhow};
use roxmltree::Document;

pub async fn get_locale_data(language: &str, region: &str) -> Result<Model> {
    let tree_set_str = TreeSetStr::get(language, region).await?;
    from_xml(region, &tree_set_str)
}

struct TreeSetStr {
    suplemental_data: String,
    main_language: String,
    main_region: String,
    main_root: String,
}

impl TreeSetStr {
    async fn get(language: &str, region: &str) -> Result<Self> {
        let ln_url = format!(
            "https://raw.githubusercontent.com/unicode-org/cldr/refs/heads/main/common/main/{language}.xml"
        );
        let reg_url = format!(
            "https://raw.githubusercontent.com/unicode-org/cldr/refs/heads/main/common/main/{language}-{region}.xml"
        );

        const SUPPLEMENTAL_URL: &str = "https://raw.githubusercontent.com/unicode-org/cldr/refs/heads/main/common/supplemental/supplementalData.xml";
        const MAIN_ROOT_URL: &str = "https://github.com/unicode-org/cldr/raw/96e54b273d2da4ec6fbad350ba19ff23fa24a588/common/main/root.xml";

        Ok(TreeSetStr {
            suplemental_data: get_text(SUPPLEMENTAL_URL).await?,
            main_language: get_text(&ln_url).await?,
            main_region: get_text(&reg_url).await?,
            main_root: get_text(MAIN_ROOT_URL).await?,
        })
    }

    fn parse<'a>(&'a self) -> Result<TreeSetDoc<'a>> {
        Ok(TreeSetDoc {
            suplemental_data: Document::parse(&self.suplemental_data)?,
            main_region: Document::parse(&self.main_region)?,
            main_language: Document::parse(&self.main_language)?,
            main_root: Document::parse(&self.main_root)?,
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
            main: BackedNode {
                main: XmlNode(self.main_language.root_element()),
                back: XmlNode(self.main_region.root_element()),
            },
            main_root: XmlNode(self.main_root.root_element()),
        }
    }
}

struct TreeSet<'a> {
    region: &'a str,
    suplemental_data: XmlNode<'a, 'a>,
    main: BackedNode<'a>,
    main_root: XmlNode<'a, 'a>,
}

impl<'a> TreeSet<'a> {
    fn get_currency_id(&self) -> Result<&str> {
        self.suplemental_data
            .select(&format!(
                "currencyData > region[iso3166={}] > currency:first-child",
                self.region
            ))
            .ok_or(anyhow!("Failed to find"))?
            .attribute("iso4217")
            .ok_or(anyhow!("Couldn't find currency name"))
    }
}

struct XmlNode<'a, 'input: 'a>(roxmltree::Node<'a, 'input>);

impl<'a, 'input: 'a> XmlNode<'a, 'input> {
    fn select(&self, text: &str) -> Option<roxmltree::Node<'a, 'input>> {
        let selectors = simplecss::Selector::parse(text)?;
        self.0
            .descendants()
            .filter(|n| n.is_element())
            .find(|&node| selectors.matches(&XmlNode(node)))
    }
}

impl simplecss::Element for XmlNode<'_, '_> {
    fn parent_element(&self) -> Option<Self> {
        self.0.parent_element().map(XmlNode)
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.0
            .prev_siblings()
            .filter(|n| n.is_element())
            .nth(0)
            .map(XmlNode)
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

/// A node with a fallback, when searching first we search in the main one and then,
/// we search on the fallback.
struct BackedNode<'a> {
    main: XmlNode<'a, 'a>,
    back: XmlNode<'a, 'a>,
}

impl<'a> BackedNode<'a> {
    fn select(&self, text: &str) -> Option<roxmltree::Node<'a, 'a>> {
        self.main.select(text).or_else(|| self.back.select(text))
    }
}

fn from_xml(region: &str, set: &TreeSetStr) -> Result<Model> {
    let tree_set_doc = set.parse()?;
    let tree_set = tree_set_doc.get_roots(region);

    Ok(Model {
        time: extract_time(&tree_set)?,
        numeric: extract_numeric(&tree_set)?,
        monetary: extract_monetary(&tree_set)?,
        messages: ModelMessages::default(),
    })
}

fn extract_time(tree_set: &TreeSet<'_>) -> Result<ModelTime> {
    Ok(ModelTime {
        abday: todo!(),
        day: todo!(),
        abmon: todo!(),
        mon: todo!(),
        am_pm: todo!(),
        d_t_fmt: todo!(),
        d_fmt: todo!(),
        t_fmt: todo!(),
        t_fmt_ampm: todo!(),
        era: todo!(),
        era_d_t_fmt: todo!(),
        era_d_fmt: todo!(),
        era_t_fmt: todo!(),
        alt_digits: todo!(),
    })
}

fn extract_numeric(tree_set: &TreeSet<'_>) -> Result<ModelNumeric> {
    Ok(ModelNumeric {
        decimal_point: todo!(),
        thousands_sep: todo!(),
        grouping: todo!(),
    })
}

fn extract_monetary(tree_set: &TreeSet<'_>) -> Result<ModelMonetary> {
    fn int_curr_symbol(tree_set: &TreeSet<'_>) -> Result<String> {
        let currency_id = tree_set.get_currency_id()?;

        assert!(currency_id.len() == 3);

        let currency_separator = tree_set
            .main
            .select("numbers > currencyFormats > currencyFormatLength:first-child > currencyFormat[alt='alphaNextToNumber'] > pattern")
            .and_then(|n|n.text()).unwrap_or(" ");

        assert!(currency_separator.len() == 1);

        Ok(format!("{}{}", currency_id, currency_separator))
    }

    fn currency_symbol(tree_set: &TreeSet<'_>) -> Result<String> {
        let currency_id = tree_set.get_currency_id()?;

        tree_set
            .main_root
            .select(&format!(
                "numbers > currencies > currency[type='{}'] > symbol",
                currency_id
            ))
            .ok_or(anyhow!(""))?
            .text()
            .ok_or(anyhow!("Currency node had no content"))
            .map(str::to_owned)
    }

    Ok(ModelMonetary {
        int_curr_symbol: int_curr_symbol(tree_set)?,
        currency_symbol: currency_symbol(tree_set)?,
        mon_decimal_point: todo!(),
        mon_thousands_sep: todo!(),
        mon_grouping: todo!(),
        positive_sign: todo!(),
        negative_sign: todo!(),
        int_frac_digits: todo!(),
        frac_digits: todo!(),
        p_cs_precedes: todo!(),
        p_sep_by_space: todo!(),
        n_cs_precedes: todo!(),
        n_sep_by_space: todo!(),
        p_sign_posn: todo!(),
        n_sign_posn: todo!(),
        int_p_cs_precedes: todo!(),
        int_p_sep_by_space: todo!(),
        int_n_cs_precedes: todo!(),
        int_n_sep_by_space: todo!(),
        int_p_sign_posn: todo!(),
        int_n_sign_posn: todo!(),
    })
}

async fn get_text(url: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url).await?.text().await
}
