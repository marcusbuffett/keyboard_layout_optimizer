use keyboard_layout::{
    config::LayoutConfig, keyboard::Keyboard, layout::Layout, layout_generator::NeoLayoutGenerator,
};
use layout_evaluation::{
    config::EvaluationParameters,
    evaluation::Evaluator,
    ngram_mapper::on_demand_ngram_mapper::OnDemandNgramMapper,
    ngrams::{Bigrams, Trigrams, Unigrams},
};

use clap::Parser;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[clap(name = "Keyboard layout evaluation")]
pub struct Options {
    /// Path to ngram files
    #[clap(
        short,
        long,
        default_value = "corpus/deu_mixed_wiki_web_0.6_eng_news_typical_wiki_web_0.4"
    )]
    pub ngrams: String,

    /// Filename of evaluation configuration file to use
    #[clap(short, long, default_value = "config/evaluation_parameters.yml")]
    pub eval_parameters: String,

    /// Filename of layout configuration file to use
    #[clap(short, long, default_value = "config/standard_keyboard.yml")]
    pub layout_config: String,

    /// Filename of corpus file to use instead of ngram files
    #[clap(short, long)]
    pub corpus: Option<String>,

    /// Evaluate given text instead of corpus file or ngram files
    #[clap(short, long)]
    pub text: Option<String>,

    /// Only consider the top ngrams up to the given fraction
    #[clap(long)]
    pub tops: Option<f64>,

    /// Do not split modifiers
    #[clap(long)]
    pub no_split_modifiers: bool,

    /// Do not add secondary bigrams from trigrams
    #[clap(long)]
    pub no_add_secondary_bigrams: bool,

    /// Do not increase weight of common bigrams
    #[clap(long)]
    pub no_increase_common_bigrams: bool,
}

#[derive(Parser, Debug)]
#[clap(name = "Keyboard layout publication")]
pub struct PublishingOptions {
    /// Publish found layout to webservice under this name.
    /// This option is required if you want to publish your layout(s)!
    #[clap(long)]
    pub publish_as: Option<String>,

    /// Publish the layout only if its cost is lower (better) than this value
    #[clap(long, requires = "publish-as")]
    pub publish_if_cost_below: Option<f64>,

    /// Publish found layout to webservice for this layout config
    #[clap(long, default_value = "standard")]
    pub publish_layout_config: String,

    /// Publish found layout to webservice at this url
    #[clap(
        long,
        default_value = "https://keyboard-layout-optimizer.herokuapp.com/api"
    )]
    pub publish_to: String,
}

pub fn init(options: &Options) -> (NeoLayoutGenerator, Evaluator) {
    (
        init_layout_generator(&options.layout_config),
        init_evaluator(options),
    )
}

pub fn init_layout_generator(layout_config: &str) -> NeoLayoutGenerator {
    let layout_config = LayoutConfig::from_yaml(layout_config)
        .expect(&format!("Could not load config file {}", layout_config));

    let keyboard = Arc::new(Keyboard::from_yaml_object(layout_config.keyboard));

    NeoLayoutGenerator::from_object(layout_config.base_layout, keyboard)
}

pub fn init_evaluator(options: &Options) -> Evaluator {
    let eval_params = EvaluationParameters::from_yaml(&options.eval_parameters).expect(&format!(
        "Could not read evaluation yaml file {}",
        options.eval_parameters
    ));

    let text = options.text.as_ref().cloned().or_else(|| {
        options.corpus.as_ref().map(|corpus_file| {
            std::fs::read_to_string(&corpus_file)
                .expect(&format!("Could not read corpus file from {}.", corpus_file))
        })
    });

    let mut ngram_mapper_config = eval_params.ngram_mapper.clone();
    if options.no_split_modifiers {
        ngram_mapper_config.split_modifiers.enabled = false;
    }
    if options.no_add_secondary_bigrams {
        ngram_mapper_config.secondary_bigrams_from_trigrams.enabled = false;
    }
    if options.no_increase_common_bigrams {
        ngram_mapper_config.increase_common_bigrams.enabled = false;
    }

    let ngram_provider = match text {
        Some(txt) => OnDemandNgramMapper::with_corpus(&txt, ngram_mapper_config),
        None => {
            let p = Path::new(&options.ngrams).join("1-grams.txt");
            log::info!("Reading unigram file: '{:?}'", p);
            let mut unigrams = Unigrams::from_file(&p.to_str().unwrap())
                .expect(&format!("Could not read 1-gramme file from '{:?}'.", &p));
            let p = Path::new(&options.ngrams).join("2-grams.txt");
            log::info!("Reading bigram file: '{:?}'", p);
            let mut bigrams = Bigrams::from_file(&p.to_str().unwrap())
                .expect(&format!("Could not read 2-gramme file from '{:?}'.", &p));
            let p = Path::new(&options.ngrams).join("3-grams.txt");
            log::info!("Reading trigram file: '{:?}'", p);
            let mut trigrams = Trigrams::from_file(&p.to_str().unwrap())
                .expect(&format!("Could not read 3-gramme file from '{:?}'.", &p));

            if let Some(tops) = options.tops {
                unigrams = unigrams.tops(tops);
                bigrams = bigrams.tops(tops);
                trigrams = trigrams.tops(tops);
            }

            OnDemandNgramMapper::with_ngrams(unigrams, bigrams, trigrams, ngram_mapper_config)
        }
    };

    Evaluator::default(Box::new(ngram_provider)).default_metrics(&eval_params.metrics)
}

/// Appends a layout-string to a file.
pub fn append_to_file(layout: &Layout, filename: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .unwrap();
    if let Err(e) = writeln!(file, "{}", layout.as_text()) {
        log::error!("Couldn't write to file: {}", e);
    } else {
        log::info!("Appended layout '{}' to '{}'", layout.as_text(), filename);
    }
}

/// Publishes the layout to a webservice.
pub fn publish_to_webservice(
    layout: &Layout,
    publish_name: &str,
    publish_to: &str,
    publish_layout_config: &str,
) {
    let client = reqwest::blocking::Client::new();
    let mut body = HashMap::new();
    body.insert("published_by", publish_name.to_string());
    body.insert("layout", layout.as_text());
    body.insert("layout_config", publish_layout_config.to_string());

    let resp = client.post(publish_to).json(&body).send().ok();
    if let Some(resp) = resp {
        if resp.status().is_success() {
            log::info!("Published layout '{}' to {}", layout.as_text(), publish_to);
        } else {
            log::error!("Could not publish result to webservice: {:?}", &resp.text());
        }
    } else {
        log::error!("Could not publish result to webservice");
    }
}
