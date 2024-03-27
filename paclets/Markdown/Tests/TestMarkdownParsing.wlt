Needs["ConnorGray`Markdown`"]

VerificationTest[
	MarkdownParse["hello"]
	,
	{MarkdownElement["Paragraph", {MarkdownElement["Text", "hello"]}]}
]

VerificationTest[
	MarkdownParse["*hello*"]
	,
	{MarkdownElement["Paragraph", {
		MarkdownElement["Emphasis", {
			MarkdownElement["Text", "hello"]
		}]
	}]}
]

VerificationTest[
	MarkdownParse["**hello**"]
	,
	{MarkdownElement["Paragraph", {
		MarkdownElement["Strong", {
			MarkdownElement["Text", "hello"]
		}]
	}]}
]

VerificationTest[
	MarkdownParse["*hello* **world**"]
	,
	{MarkdownElement["Paragraph", {
		MarkdownElement["Emphasis", {MarkdownElement["Text", "hello"]}],
		MarkdownElement["Text", " "],
		MarkdownElement["Strong", {MarkdownElement["Text", "world"]}]
	}]}
]

VerificationTest[
	MarkdownParse["**`code`**"]
	,
	{MarkdownElement["Paragraph", {
		MarkdownElement["Strong", {
			MarkdownElement["Code", "code"]
		}]
	}]}
]

VerificationTest[
	MarkdownParse["_**hello**_"]
	,
	{MarkdownElement["Paragraph", {
		MarkdownElement["Emphasis", {
			MarkdownElement["Strong", {
				MarkdownElement["Text", "hello"]
			}]
		}]
	}]}
]

VerificationTest[
	MarkdownParse["* one\n* two\n* three"]
	,
	{
		MarkdownElement[
			"List",
			{
				MarkdownElement["ListItem", {
					MarkdownElement["Paragraph", {MarkdownElement["Text", "one"]}]
				}],
				MarkdownElement["ListItem", {
					MarkdownElement["Paragraph", {MarkdownElement["Text", "two"]}]
				}],
				MarkdownElement["ListItem", {
					MarkdownElement["Paragraph", {MarkdownElement["Text", "three"]}]
				}]
			}
		]
	}
]

VerificationTest[
	MarkdownParse@"
This is a paragraph
with a soft line break.

> This is a block quote
>
> With *multiple* statements
	",
	{
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "This is a paragraph"],
			MarkdownElement["SoftBreak"],
			MarkdownElement["Text", "with a soft line break."]
		}],
		MarkdownElement["BlockQuote", {
			MarkdownElement["Paragraph", {
				MarkdownElement["Text", "This is a block quote"]
			}],
			MarkdownElement["Paragraph", {
				MarkdownElement["Text", "With "],
				MarkdownElement["Emphasis", {
					MarkdownElement["Text", "multiple"]
				}],
				MarkdownElement["Text", " statements"]
			}]
		}]
	}
]

VerificationTest[MarkdownParse@"
* Hello how are *you*

  The quick brown fox jumps over the lazy dog.\\
  And then it lays down for a nap.

* This is the second list element
	",
	{
		MarkdownElement["List", {
			MarkdownElement["ListItem", {
				MarkdownElement["Paragraph", {
					MarkdownElement["Text", "Hello how are "],
					MarkdownElement["Emphasis", {
						MarkdownElement["Text", "you"]
					}]
				}],
				MarkdownElement["Paragraph", {
					MarkdownElement["Text", "The quick brown fox jumps over the lazy dog."],
					MarkdownElement["HardBreak"],
					MarkdownElement["Text", "And then it lays down for a nap."]
				}]
			}],
			MarkdownElement["ListItem", {
				MarkdownElement["Paragraph", {
					MarkdownElement["Text", "This is the second list element"]
				}]
			}]
		}]
	}
]
