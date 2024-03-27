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
	MarkdownParse["`foo` is a **function**"]
	,
	{MarkdownElement["Paragraph", {
		MarkdownElement["Code", "foo"],
		MarkdownElement["Text", " is a "],
		MarkdownElement["Strong", {
			MarkdownElement["Text", "function"]
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
	MarkdownParse["
# This is an H1

## This is an H2

###### This is an H6

######## This is too many levels!
"]
	,
	{
		MarkdownElement["Heading", 1, {MarkdownElement["Text", "This is an H1"]}],
		MarkdownElement["Heading", 2, {MarkdownElement["Text", "This is an H2"]}],
		MarkdownElement["Heading", 6, {MarkdownElement["Text", "This is an H6"]}],
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "######## This is too many levels!"]
		}]
	}
]

VerificationTest[
	MarkdownParse["
The content before.
*****
The content after.
"]
	,
	{
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "The content before."]
		}],
		MarkdownElement["ThematicBreak"],
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "The content after."]
		}]
	}
]

(* Test code blocks *)
VerificationTest[
	MarkdownParse["
This is a code block:

```wolfram
f[g[h[]]]
```

This is an indented code block:

    could_be_any_language()
"]
	,
	{
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "This is a code block:"]
		}],
		MarkdownElement["CodeBlock", "wolfram", "f[g[h[]]]\n"],
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "This is an indented code block:"]
		}],
		MarkdownElement["CodeBlock", None, "could_be_any_language()\n"]
	}
]

(*====================================*)
(* Composite tests                    *)
(*====================================*)

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

(* Test thematic breaks in block quotes *)
VerificationTest[
	MarkdownParse["
> This is earlier in the story.
> *****
> This is later in the story.
"]
	,
	{
		MarkdownElement["BlockQuote", {
			MarkdownElement["Paragraph", {
				MarkdownElement["Text", "This is earlier in the story."]
			}],
			MarkdownElement["ThematicBreak"],
			MarkdownElement["Paragraph", {
				MarkdownElement["Text", "This is later in the story."]
			}]
		}]
	}
]

(* Test thematic breaks in list items *)
VerificationTest[
	MarkdownParse["
1. This is a complicated list item.
   *****
   It has multiple thematic pieces!

2. And that's not even the only item in this list!
"]
	,
	{
		MarkdownElement["List", {
			MarkdownElement["ListItem", {
				MarkdownElement["Paragraph", {
					MarkdownElement["Text", "This is a complicated list item."]
				}],
				MarkdownElement["ThematicBreak"],
				MarkdownElement["Paragraph", {
					MarkdownElement["Text", "It has multiple thematic pieces!"]
				}]
			}],
			MarkdownElement["ListItem", {
				MarkdownElement["Paragraph", {
					MarkdownElement["Text", "And that's not even the only item in this list!"]
				}]
			}]
		}]
	}
]