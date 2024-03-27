Needs["ConnorGray`Markdown`"]

VerificationTest[
	ConvertToMarkdown @ Notebook[{
		Cell["Hello Markdown!", "Title"],

		Cell["This is a Section", "Section"],

		Cell["This is some plain text.", "Text"]
	}],
	{
		MarkdownElement["Heading", 1, {
			MarkdownElement["Text", "Hello Markdown!"]
		}],
		MarkdownElement["Heading", 3, {
			MarkdownElement["Text", "This is a Section"]
		}],
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "This is some plain text."]
		}]
	}
]

VerificationTest[
	ConvertToMarkdown @ Notebook[{
		Cell["This is a code block:", "Text"],
		Cell["println!(\"Hello, World!\");", "Program"]
	}],
	{
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "This is a code block:"]
		}],
		MarkdownElement["CodeBlock", None,
			"println!(\"Hello, World!\");"
		]
	}
]