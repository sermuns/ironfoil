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

#let download-button(image-path, os-name) = {
  set page(
    width: auto,
    height: auto,
    margin: 0pt,
    fill: none,
  )

  set text(
    font: "New Computer Modern Sans",
    top-edge: "bounds",
    bottom-edge: "bounds",
    fill: white,
  )

  set align(center + horizon)

  box(
    width: 6em,
    height: 1.5em,
    fill: blue.darken(60%).transparentize(15%),
    radius: .2em,
    stack(
      dir: ltr,
      spacing: .3em,
      image(
        bytes(
          read(image-path).replace(
            "#000000",
            foreground-dark.to-hex(),
          ),
        ),
        height: 1em,
      ),
      os-name,
    ),
  )
}
