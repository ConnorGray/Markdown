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