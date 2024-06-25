Needs["ConnorGray`Markdown`"]

VerificationTest[
	MarkdownToNotebook @ {
		MarkdownElement["Heading", 1, {
			MarkdownElement["Text", "Main Heading"]
		}],
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "Some plain text."]
		}],
		MarkdownElement["CodeBlock", "rust", "fn foo() {\n    2 + 2\n}"]
	},
	Notebook[{
		Cell[TextData[{"Main Heading"}], "Title"],
		Cell[TextData[{"Some plain text."}], "Text"],
		Cell[
			"fn foo() {\n    2 + 2\n}",
			"Program",
			TaggingRules -> <| "ConnorGray/Markdown" -> <|
				"CodeBlockInfoString" -> "rust"
			|>|>
		]
	}, StyleDefinitions -> FrontEnd`FileName[
		{"ConnorGray"},
		"Markdown.nb",
		CharacterEncoding -> "UTF-8"
	]]
]

(* TID:240625/1: Converting inline styles to cells *)
VerificationTest[
	First @ MarkdownToNotebook @ MarkdownParse @ StringJoin[
		"This `Markdown` contains **bold** and *italic* text."
	],
	{
		Cell[
			TextData[{
				"This ",
				StyleBox["Markdown", "Code"],
				" contains ",
				StyleBox[{"bold"}, FontWeight -> "Bold"],
				" and ",
				StyleBox[{"italic"}, FontSlant -> "Italic"],
				" text."
			}],
			"Text"
		]
	}
]