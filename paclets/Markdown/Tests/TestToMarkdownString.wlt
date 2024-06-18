Needs["ConnorGray`Markdown`"]

VerificationTest[
	ToMarkdownString @ {
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "hello"]
		}]
	},
	"hello"
]

VerificationTest[
	ToMarkdownString @ {
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "hello "],
			MarkdownElement["Strong", {
				MarkdownElement["Text", "world"]
			}]
		}]
	},
	"hello **world**"
]

VerificationTest[
	ToMarkdownString @ {
		MarkdownElement["Heading", 1, {
			MarkdownElement["Text", "This is a heading."]
		}],
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "This is some text."]
		}]
	},
	"\
# This is a heading.

This is some text."
]