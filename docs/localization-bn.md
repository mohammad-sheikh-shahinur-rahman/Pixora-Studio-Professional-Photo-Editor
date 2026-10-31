# Bengali (বাংলা) terminology and catalog notes

This note accompanies the Bengali catalog (`crates/ui-egui/src/i18n/bn.tsv`), registered in
[i18n `LANGUAGES`](../crates/ui-egui/src/i18n/mod.rs) as `bn` (native name বাংলা, plural rule
one/other like English, `complete_menus: true`) and covering every key
`cargo xtask i18n-coverage` measures. Additions follow `docs/localization-ru.md`'s precedent:
translations are written from the meaning of Pixora Studio's English source keys, reusing the
catalog's own established vocabulary, and no proprietary translation resources were consulted.

## Register and style

- **Commands are imperatives in the polite second person** (আপনি register): সংরক্ষণ করুন, মুছুন,
  বাইকিউবিক (স্বয়ংক্রিয়). Status and dialog sentences end in হয়েছে/করুন/ছে/না as natural Bengali.
- **Ellipses on dialog commands are preserved** অন্য নামে সংরক্ষণ…, and `{name}` placeholders and
  line breaks appear in both columns (translators may reorder placeholders freely).
- **Digits:** UI texts use Bengali numerals (৫০% থ্রেশহোল্ড, ২৫৬ × ২৫৬ পিক্সেল, ৩ × ৩) except where
  the surrounding token is a technical literal that the code or a file name compares against
  (`100%`, `180°`, units such as `MB`, `.kys`, `256 × 256` inside the ICO sentence keep Latin
  digits, as simplified-Chinese does for cases where the value is a file-format literal).
- **Punctuation** follows the English layout (the character `:` and parentheses stay ASCII where
  the English shell command or key name is inline, e.g. `say`); Bengali full stop । is not used,
  the Western dot stays so that interpolated file extensions read unambiguously.
- Bengali has no case; the colon, dash and of-genitive (ঢক্কন-র) spacing rules follow the catalog's
  first occurrence of each pattern so grep-friendly parallel forms stay consistent.
- `@plural` rows carry the two English forms and **two Bengali forms** (`one|other`), joined the
  same way as the source: `{n}টি অ্যাকশন|{n}টি অ্যাকশন` when the noun phrase stays identical
  (Bengali does not inflect the noun for number).

## Terminology

Choices the catalog introduced, to keep the same English term reading the same everywhere:

| English source | Bengali | Note |
|---|---|---|
| Layer | লেয়ার | transliterated everywhere |
| Channel | চ্যানেল | Channels → চ্যানেলগুলোর প্যানেল প্রসঙ্গে অনুবাদ |
| Mask | মাস্ক | Layer Mask → লেয়ার মাস্ক; Flatten All Masks → সব মাস্ক সমতল |
| Smart Object | স্মার্ট অবজেক্ট | transliterated |
| Artboard | আর্টবোর্ড | আর্টবোর্ড, আর্টবোর্ড গাইড |
| Guide | গাইড | Guides, Grid & Slices → গাইড, গ্রিড ও স্লাইস |
| Feather | ফেদার | never পালকীকরণ; also in the brush rows |
| Highlight(s) / Midtone(s) / Shadow(s) | হাইলাইট / মিডটোন / ছায়া plus ক্ষেণ | Levels, Curves and Shadows/Highlights reuse them |
| Noise Reduction | নয়েজ হ্রাস | Camera Raw vocabulary |
| Clarity / Vibrance / Saturation | ক্ল্যারিটি / ভাইব্র্যান্স / স্যাচুরেশন | transliterated |
| Content-Aware Fill | কনটেন্ট-অ্যাওয়্যার ফিল | feature keeps its English name's meaning |
| Rasterize | র‍্যাস্টারাইজ | ZWNJ preserved in র‍্যাস্টারাইজ |
| Save As… | অন্য নামে সংরক্ষণ… | distinct from Save (সংরক্ষণ) and Export (রপ্তানি) |
| Export | রপ্তানি | Import is আমদানি |
| Replace | প্রতিস্থাপন করুন | the overwrite prompt's verb |
| Preset | প্রিসেট | প্রিসেট ম্যানেজার… |
| Amount | পরিমাণ | strengths and amounts alike |

Blend modes are transliterated (মালটিপ্লাই, স্ক্রিন, ওভারলে), matching Chinese/Italian's example of
naming every mode in its own script. Product, file-format and colour-space names keep their
Latin spelling (Pixora Studio, PSD, JPEG, OpenEXR, AVIF, PDF, DDS, EXIF, LUT, RGB, CMYK, Lab,
HDR, ACEScg) as do theme names (Pro, Studio, Solarized, Breeze) and acronyms. Commands that name
tools (L, M, V) show those keys untranslated, they are the same in every language.

## Not covered

Engine error and status messages, user document, layer and preset names, and automation command
IDs stay in English, as in every language ([`localization-parity.md`](localization-parity.md)).

## Native review

The catalog was written AI-assisted from the English source's meaning. A review by a native
Bengali speaker is still welcome, especially of the filter-parameter rows and the
shortcut-proofing sentences where idiom matters most.

## Validation

- `cargo xtask i18n-coverage`: `bn` 2060/2060 (100%); `complete_menus: true` in
  `crates/ui-egui/src/i18n/mod.rs`.
- `cargo test -p photocraft-ui-egui`: the catalog parse, placeholder and plural-form tests pass
  for every registered language including `bn`.
- `cargo xtask layers` and the `wasm32-unknown-unknown` check stay green: the catalog is data
  packed by `build.rs`, no new dependencies.
- The offscreen `snapshot` example with `PHOTOCRAFT_LOCALE=bn` renders with Bengali text and no
  missing glyphs (font stack covers Bengali; report any missing-glyph warnings here).
