#![cfg(feature = "html")]

use jumpcut::{parse, rendering::html::HtmlRenderOptions};
use std::{fs, process::Command};

/// Run with CHROME_BIN pointing to Chrome/Chromium:
/// cargo test --test html_transition_layout_test -- --ignored
/// This measures rendered text, including the final colon, rather than just
/// asserting that a particular CSS declaration is present.
#[test]
#[ignore = "requires Chrome/Chromium via CHROME_BIN"]
fn transitions_match_pdf_position_relative_to_action_text_in_every_html_mode() {
    let chrome = std::env::var("CHROME_BIN").expect("set CHROME_BIN to Chrome/Chromium");
    let directory =
        std::env::temp_dir().join(format!("jumpcut-html-layout-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let mut screenplay = parse(
        "INT. TUNNELS - GIANTS STADIUM - MOMENTS LATER\n\nDanny rushes to the trash can, but the bag's been changed.\n\nCUT TO:\n\nDISSOLVE TO:\n\n> **SMASH CUT TO:**\n\n> A MUCH LONGER TRANSITION THAT WRAPS TO:\n",
    );

    for (mode, exact_wraps, paginated) in [
        ("normal", false, false),
        ("exact", true, false),
        ("paginated", false, true),
    ] {
        let mut html = screenplay.to_html_with_options(HtmlRenderOptions {
            exact_wraps,
            paginated,
            embed_courier_prime: true,
            ..HtmlRenderOptions::default()
        });
        html.push_str(
            r#"<pre id="layout-result">pending</pre>
<script>
document.fonts.ready.then(() => {
  const result = document.getElementById('layout-result');
  try {
    const screenplay = document.querySelector('.screenplay');
    const action = screenplay.querySelector('.action');
    // PDF: the transition ends at 7.1in, action starts at 1.5in,
    // and each body character advances 7pt: (7.1 - 1.5) * 72 / 7 = 57.6 cells.
    // The screenshot's action ends in "changed." at cell 58. The colon should
    // end 0.4 cells before it, NOT at the HTML container's right edge.
    const reference = document.createRange();
    reference.setStart(action.firstChild, 0);
    reference.setEnd(action.firstChild, 1);
    const cellWidth = reference.getBoundingClientRect().width;
    if (Math.abs(cellWidth - 7 * 96 / 72) > 0.1) {
      throw new Error(`Body advance ${cellWidth}px does not match PDF's 7pt grid`);
    }
    const transitions = [...screenplay.querySelectorAll('.transition')];
    if (transitions.length < 4) throw new Error('Missing transition fixtures');
    if (!transitions.some(t => t.querySelector('.bold'))) {
      throw new Error('Missing styled transition fixture');
    }
    // Ordinary HTML is responsive; exact/paginated output uses fixed wraps.
    const widths = screenplay.classList.contains('exactWraps') ? [null] : [576, 420];
    for (const width of widths) {
      if (width) screenplay.style.width = `${width}px`;
      const expected = Math.min(action.getBoundingClientRect().right,
        action.getBoundingClientRect().left + 5.6 * 96);
      for (const transition of transitions) {
        const walker = document.createTreeWalker(transition, NodeFilter.SHOW_TEXT);
        let lastNode, lastOffset;
        while (walker.nextNode()) {
          const end = walker.currentNode.textContent.trimEnd().length;
          if (end) { lastNode = walker.currentNode; lastOffset = end; }
        }
        const range = document.createRange();
        range.setStart(lastNode, lastOffset - 1);
        range.setEnd(lastNode, lastOffset);
        const actual = range.getBoundingClientRect().right;
        if (Math.abs(actual - expected) > 1) {
          throw new Error(`${transition.textContent.trim()}: right edge ${actual}, expected ${expected}`);
        }
      }
    }
    result.textContent = 'PASS';
  } catch (error) {
    result.textContent = error.message;
  }
});
</script>"#,
        );
        let path = directory.join(format!("{mode}.html"));
        fs::write(&path, html).unwrap();
        let output = Command::new(&chrome)
            .args([
                "--headless",
                "--disable-gpu",
                "--no-sandbox",
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-background-networking",
                "--window-size=1000,1000",
                "--virtual-time-budget=3000",
                "--dump-dom",
            ])
            .arg(format!(
                "--user-data-dir={}",
                directory.join("chrome").display()
            ))
            .arg(format!("file://{}", path.display()))
            .output()
            .expect("launch Chrome/Chromium");
        assert!(
            output.status.success(),
            "{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let dom = String::from_utf8(output.stdout).unwrap();
        let result = dom
            .split("<pre id=\"layout-result\">")
            .nth(1)
            .and_then(|tail| tail.split("</pre>").next())
            .expect("browser should report layout measurements");
        assert_eq!(result, "PASS", "{mode} HTML: {result}");
    }
    fs::remove_dir_all(directory).unwrap();
}
