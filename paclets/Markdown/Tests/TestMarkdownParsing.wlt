Needs["ConnorGray`Markdown`"]

VerificationTest[
	MarkdownParse["hello"]
	,
	{{Markdown`Inline["Text", "hello", {}]}}
]

VerificationTest[
	MarkdownParse["*hello*"]
	,
	{{Markdown`Inline["Text", "hello", Italic]}}
]

VerificationTest[
	MarkdownParse["**hello**"]
	,
	{{Markdown`Inline["Text", "hello", Bold]}}
]

VerificationTest[
	MarkdownParse["*hello* **world**"]
	,
	{{
		Markdown`Inline["Text", "hello", Italic],
		Markdown`Inline["Text", " ", {}],
		Markdown`Inline["Text", "world", Bold]
	}}
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
		Markdown`List[
			{{Markdown`Inline["Text", "one", {}]}},
			{{Markdown`Inline["Text", "two", {}]}},
			{{Markdown`Inline["Text", "three", {}]}}
		]
	}
]


