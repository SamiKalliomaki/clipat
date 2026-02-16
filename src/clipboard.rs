use anyhow::Context;
use arboard::Clipboard;
use once_cell::sync::Lazy;
use std::{
    borrow::Cow,
    env,
    ops::DerefMut,
    sync::{Mutex, MutexGuard},
};

#[derive(Clone)]
pub(crate) struct Image<'a> {
    pub width: usize,
    pub height: usize,
    pub bytes: Cow<'a, [u8]>,
}

#[derive(Clone)]
pub(crate) enum Content<'a> {
    Text(String),
    Image(Image<'a>),
}

static CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(None);
static FAKE_CLIPBOARD: Mutex<Option<Content<'static>>> = Mutex::new(None);
static USE_FAKE_CLIPBOARD: Lazy<bool> = Lazy::new(|| match env::var("FAKE_CLIPBOARD") {
    Ok(val) => !val.is_empty(),
    Err(_) => false,
});

fn get_clipboard<'a>(
    lock: &'a mut MutexGuard<Option<Clipboard>>,
) -> anyhow::Result<&'a mut Clipboard> {
    if lock.is_none() {
        *(lock.deref_mut()) = Some(Clipboard::new().context("Failed to open clipboard")?);
    }
    Ok(lock.as_mut().unwrap())
}

pub(crate) fn get() -> anyhow::Result<Option<Content<'static>>> {
    if *USE_FAKE_CLIPBOARD {
        let fake = FAKE_CLIPBOARD.lock().unwrap();
        return Ok(fake.clone());
    }

    let mut clipboard = CLIPBOARD.lock().unwrap();
    let clipboard = get_clipboard(&mut clipboard)?;

    if let Ok(text) = clipboard.get_text() {
        return Ok(Some(Content::Text(text)));
    }
    if let Ok(image) = clipboard.get_image() {
        return Ok(Some(Content::Image(Image {
            width: image.width,
            height: image.height,
            bytes: image.bytes,
        })));
    }

    Ok(None)
}

pub(crate) fn set(content: Content<'_>) -> anyhow::Result<()> {
    if *USE_FAKE_CLIPBOARD {
        let mut fake = FAKE_CLIPBOARD.lock().unwrap();
        match content {
            Content::Text(text) => *(fake.deref_mut()) = Some(Content::Text(text)),
            Content::Image(image) => {
                *(fake.deref_mut()) = Some(Content::Image(Image {
                    width: image.width,
                    height: image.height,
                    bytes: Cow::Owned(image.bytes.to_vec()),
                }))
            }
        }
        return Ok(());
    }

    let mut clipboard = CLIPBOARD.lock().unwrap();
    let clipboard = get_clipboard(&mut clipboard)?;

    match content {
        Content::Text(text) => clipboard.set_text(&text).context("Failed to set text"),
        Content::Image(Image {
            width,
            height,
            bytes,
        }) => clipboard
            .set_image(arboard::ImageData {
                width,
                height,
                bytes,
            })
            .context("Failed to set image"),
    }
}

pub(crate) fn clear() -> anyhow::Result<()> {
    let mut clipboard = CLIPBOARD.lock().unwrap();
    let clipboard = get_clipboard(&mut clipboard)?;
    clipboard.clear().context("Failed to clear clipboard")
}
