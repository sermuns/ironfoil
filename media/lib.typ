#let foreground-dark = white;
#let foreground-light = black;
#let stroke-color = rgb("#977e51")

#let logo(color) = {
  set text(
    size: 180pt,
    font: "New Computer Modern Sans",
  )
  stack(
    dir: ltr,
    spacing: .25em,
    image(
      bytes(
        read("logo.svg")
          .replace(
            "#000000",
            color.to-hex(),
          )
          .replace(
            "#000",
            color.to-hex(),
          ),
      ),
      height: 1.1em,
    ),
  )
}
