mod chunked_input;

use cool_thing::*;
use failure::Error;

pub use self::chunked_input::ChunkedInput;

#[derive(Copy, Clone)]
pub struct ContentSettings {
    pub document_level: DocumentLevelContentSettings,
    pub on_element_start: ContentSettingsOnElementStart,
    pub on_element_end: ContentSettingsOnElementEnd,
}

impl ContentSettings {
    pub fn all() -> Self {
        ContentSettings {
            document_level: DocumentLevelContentSettings::all(),
            on_element_start: ContentSettingsOnElementStart::CAPTURE_START_TAG_FOR_ELEMENT,
            on_element_end: ContentSettingsOnElementEnd::CAPTURE_END_TAG_FOR_ELEMENT,
        }
    }

    pub fn start_tags() -> Self {
        ContentSettings {
            document_level: DocumentLevelContentSettings::empty(),
            on_element_start: ContentSettingsOnElementStart::CAPTURE_START_TAG_FOR_ELEMENT,
            on_element_end: ContentSettingsOnElementEnd::empty(),
        }
    }

    pub fn end_tags() -> Self {
        ContentSettings {
            document_level: DocumentLevelContentSettings::empty(),
            on_element_start: ContentSettingsOnElementStart::empty(),
            on_element_end: ContentSettingsOnElementEnd::CAPTURE_END_TAG_FOR_ELEMENT,
        }
    }

    pub fn text() -> Self {
        ContentSettings {
            document_level: DocumentLevelContentSettings::CAPTURE_TEXT,
            on_element_start: ContentSettingsOnElementStart::empty(),
            on_element_end: ContentSettingsOnElementEnd::empty(),
        }
    }

    pub fn comments() -> Self {
        ContentSettings {
            document_level: DocumentLevelContentSettings::CAPTURE_COMMENTS,
            on_element_start: ContentSettingsOnElementStart::empty(),
            on_element_end: ContentSettingsOnElementEnd::empty(),
        }
    }

    pub fn doctypes() -> Self {
        ContentSettings {
            document_level: DocumentLevelContentSettings::CAPTURE_DOCTYPES,
            on_element_start: ContentSettingsOnElementStart::empty(),
            on_element_end: ContentSettingsOnElementEnd::empty(),
        }
    }
}

type TokenHandler<'h> = Box<dyn FnMut(&mut Token<'_>) + 'h>;

struct TestTransformController<'h> {
    token_handler: TokenHandler<'h>,
    content_settings: ContentSettings,
}

impl<'h> TestTransformController<'h> {
    pub fn new(token_handler: TokenHandler<'h>, content_settings: ContentSettings) -> Self {
        TestTransformController {
            token_handler,
            content_settings,
        }
    }
}

impl TransformController for TestTransformController<'_> {
    fn document_level_content_settings(&self) -> DocumentLevelContentSettings {
        self.content_settings.document_level
    }
    fn handle_element_start(&mut self, _: &TagNameInfo<'_>) -> ElementStartResponse<Self> {
        ElementStartResponse::ContentSettings(self.content_settings.on_element_start)
    }

    fn handle_element_end(&mut self, _: &TagNameInfo<'_>) -> ContentSettingsOnElementEnd {
        self.content_settings.on_element_end
    }

    fn handle_token(&mut self, token: &mut Token) {
        (self.token_handler)(token)
    }
}

pub fn parse(
    input: &ChunkedInput,
    content_settings: ContentSettings,
    initial_text_type: TextType,
    last_start_tag_name_hash: Option<u64>,
    token_handler: TokenHandler<'_>,
) -> Result<String, Error> {
    let mut output = Vec::new();

    let encoding = input
        .encoding()
        .expect("Input should be initialized before parsing");

    let transform_controller = TestTransformController::new(token_handler, content_settings);

    let mut transform_stream = TransformStream::new(
        transform_controller,
        |chunk: &[u8]| output.extend_from_slice(chunk),
        2048,
        encoding,
    );

    let parser = transform_stream.parser();

    parser.set_last_start_tag_name_hash(last_start_tag_name_hash);
    parser.switch_text_type(initial_text_type);

    for chunk in input.chunks() {
        transform_stream.write(chunk)?;
    }

    transform_stream.end()?;

    Ok(encoding.decode_without_bom_handling(&output).0.to_string())
}

macro_rules! parse_token {
    ($input:expr, $encoding:expr, $TokenType:ident, $callback:expr) => {{
        use crate::harness::parsing::{parse, ChunkedInput, ContentSettings};
        use cool_thing::{TextType, Token};

        let mut input: ChunkedInput = String::from($input).into();
        let mut emitted = false;

        input.init($encoding, true).unwrap();

        parse(
            &input,
            ContentSettings::all(),
            TextType::Data,
            None,
            Box::new(move |t| match t {
                Token::$TokenType(t) => {
                    // NOTE: we always have two text chunks:
                    // one with the actual text and the second is emitted
                    // on EOF to signify the end of the text node.
                    // We need to invoke callback only for the first one.
                    if !emitted {
                        $callback(t);
                        emitted = true;
                    }
                }
                _ => unreachable!("Input should contain only tokens of the requested type"),
            }),
        )
        .unwrap();
    }};
}