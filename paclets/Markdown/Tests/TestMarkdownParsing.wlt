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
				{MarkdownElement["Paragraph", {MarkdownElement["Text", "one"]}]},
				{MarkdownElement["Paragraph", {MarkdownElement["Text", "two"]}]},
				{MarkdownElement["Paragraph", {MarkdownElement["Text", "three"]}]}
			}
		]
	}
]


