Needs["ConnorGray`Markdown`"]

VerificationTest[
	MarkdownParse["hello"]
	,
	{MarkdownElement["Paragraph", {MarkdownElement["Text", "hello", {}]}]}
]

VerificationTest[
	MarkdownParse["*hello*"]
	,
	{MarkdownElement["Paragraph", {MarkdownElement["Text", "hello", Italic]}]}
]

VerificationTest[
	MarkdownParse["**hello**"]
	,
	{MarkdownElement["Paragraph", {MarkdownElement["Text", "hello", Bold]}]}
]

VerificationTest[
	MarkdownParse["*hello* **world**"]
	,
	{MarkdownElement["Paragraph", {
		MarkdownElement["Text", "hello", Italic],
		MarkdownElement["Text", " ", {}],
		MarkdownElement["Text", "world", Bold]
	}]}
]

(* FIXME: This is a bug, we lose the strong/bold wrapper. *)
VerificationTest[
	MarkdownParse["**`code`**"]
	,
	{MarkdownElement["Paragraph", {
		MarkdownElement["Code", "code"]
	}]}
]

(* FIXME:
    This is flaky due to use of HashSet internally causing random sorting
	of the text styling attributes.
*)
(* VerificationTest[
	MarkdownParse["_**hello**_"]
	,
	{{Markdown`Inline["Text", "hello", {Italic, Bold}]}}
] *)

VerificationTest[
	MarkdownParse["* one\n* two\n* three"]
	,
	{
		MarkdownElement[
			"List",
			{
				{MarkdownElement["Paragraph", {MarkdownElement["Text", "one", {}]}]},
				{MarkdownElement["Paragraph", {MarkdownElement["Text", "two", {}]}]},
				{MarkdownElement["Paragraph", {MarkdownElement["Text", "three", {}]}]}
			}
		]
	}
]


