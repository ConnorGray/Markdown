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

VerificationTest[
	ToMarkdownString @ {
		MarkdownElement["CodeBlock", "rust", "fn foo() -> i64 {\n    return 4;\n}"]
	},
	"
```rust
fn foo() -> i64 {
    return 4;
}
```"
]

VerificationTest[
	ToMarkdownString @ {
		MarkdownElement[
			"Heading",
			1,
			{MarkdownElement["Text", "Example Document"]}
		],
		MarkdownElement["Paragraph",{
			MarkdownElement["Text", "This is some text"]
		}],
		MarkdownElement["CodeBlock", "wolfram,cell:Input", "foo[x_] := x + 1"],
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "This is "],
			MarkdownElement["Strong", {MarkdownElement["Text", "some "]}],
			MarkdownElement[
				"Emphasis",
				{MarkdownElement["Strong", {MarkdownElement["Text", "styled"]}]}
			],
			MarkdownElement["Strong", {MarkdownElement["Text", " text"]}],
			MarkdownElement["Text", "."]
		}]
	},
	"\
# Example Document

This is some text

```wolfram,cell:Input
foo[x_] := x + 1
```

This is **some *****styled***** text**."
]