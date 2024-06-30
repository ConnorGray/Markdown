Needs["ConnorGray`Markdown`"]

Needs["Wolfram`ErrorTools`"]

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

(* TID:240625/2: Converting softbreaks to cells *)
VerificationTest[
	First @ MarkdownToNotebook @ MarkdownParse @ StringRiffle[{
		"This is one Markdown paragraph",
		"split across multiple lines"
	}, "\n"],
	{
		Cell[
			TextData[{
				"This is one Markdown paragraph",
				"\n",
				"split across multiple lines"
			}],
			"Text"
		]
	}
]

(*========================================================*)
(* Test conversion of lists                               *)
(*========================================================*)

(* TID:240625/3: Converting simple lists to cells *)
VerificationTest[
	First @ MarkdownToNotebook @ MarkdownParse @ StringRiffle[{
		"* This is a Markdown list",
		"* With multiple list items"
	}, "\n"],
	{
		Cell[
			TextData[{"This is a Markdown list"}],
			"Item"
		],
		Cell[
			TextData[{"With multiple list items"}],
			"Item"
		]
	}
]

(* TID:240625/4: Converting multi-block list items to cells *)
VerificationTest[
	First @ MarkdownToNotebook @ MarkdownParse @ StringRiffle[{
		"* This is a multi-block Markdown list item.",
		"",
		"  It contains multiple paragraphs.",
		"* And the overall list contains multiple list items."
	}, "\n"],
	{
		Cell[
			TextData[{"This is a multi-block Markdown list item."}],
			"Item"
		],
		Cell[
			TextData[{"It contains multiple paragraphs."}],
			"ItemParagraph"
		],
		Cell[
			TextData[{"And the overall list contains multiple list items."}],
			"Item"
		]
	}
]

(* TID:240627/1: Error for over-nested list item. *)
VerificationTest[
	MarkdownToNotebook @ MarkdownParse @ StringRiffle[{
		"* First level",
		"  * Second level",
		"    * Third level",
		"      * Fourth level"
	}, "\n"],
	Failure[MarkdownError, <|
		"CausedBy" -> Failure[MarkdownError, <|
			"CausedBy" -> Failure[MarkdownError, <|
				"MessageTemplate" -> "Unsupported nested list depth beyond 3: ``",
				"MessageParameters" -> {InputForm[4]}
			|>],
			"MessageTemplate" -> "Error converting Paragraph element: ``",
			"MessageParameters" -> {
				InputForm @ MarkdownElement[
					"Paragraph",
					{MarkdownElement["Text", "Fourth level"]}
				]
			}
		|>],
		"MessageTemplate" -> "Error converting Markdown to notebook",
		"MessageParameters" -> {}
	|>]
]

(* TID:240630/1: Error for expected List children to be ListItem. *)
VerificationTest[
	MarkdownToNotebook @ MarkdownElement["List", {
		MarkdownElement["Paragraph", {
			MarkdownElement["Text", "This is the first list item"]
		}]
	}],
	Failure[MarkdownError, <|
		"CausedBy" -> Failure[MarkdownError, <|
			"MarkdownElement" -> MarkdownElement["List", {
				MarkdownElement["Paragraph", {
					MarkdownElement["Text", "This is the first list item"]
				}]
			}],
			"MessageTemplate" -> "Expected \"List\" element children to be \"ListItem\" elements.",
			"MessageParameters" -> {}
		|>],
		"MessageTemplate" -> "Error converting Markdown to notebook",
		"MessageParameters" -> {}
	|>]
]

(*====================================*)

(*
	Tests for elements not supported not at the document root
*)

(* TID:240625/5: Heading not supported inside list item *)
VerificationTest[
	MarkdownToNotebook @ MarkdownParse @ StringRiffle[{
		"* This is a multi-block Markdown list item.",
		"",
		"  ### It Has a Heading",
		"",
		"  And lots of blocks."
	}, "\n"],
	Failure[MarkdownError, <|
		"CausedBy" -> Failure[MarkdownError, <|
			"MarkdownElement" -> MarkdownElement["Heading", 3, {
				MarkdownElement["Text", "It Has a Heading"]
			}],
			"CurrentBlockStack" -> {"List", {"ListItem", 2}},
			"MessageTemplate" -> "Unsupported `` type Markdown element not at document root",
			"MessageParameters" -> {InputForm["Heading"]}
		|>],
		"MessageTemplate" -> "Error converting Markdown to notebook",
		"MessageParameters" -> {}
	|>]
]

(* TID:240625/6: Heading not supported inside blockquote *)
(* TODO: Support block quotes and re-enable this test. *)
(* VerificationTest[
	MarkdownToNotebook @ MarkdownParse @ StringRiffle[{
		"> This is a Markdown blockquote.\n",
		"> ",
		"> ### It Has a Heading",
		"> ",
		"> And lots of blocks."
	}, "\n"],
	{
	}
] *)

(*========================================================*)

(*
	Test that even after all of the above tests, $nestedBlockStack is still
	dynamically scoped back to its original error value. It hasn't been
	erroneously unset outside of a Block[..].
*)
VerificationTest[
	Handle[_Failure][
		ConnorGray`Markdown`MarkdownToNotebook`Private`$nestedBlockStack
	],
	Failure[MarkdownError, <|
		"MessageTemplate" -> "Invalid unscoped attempt to access $nestedBlockStack",
		"MessageParameters" -> {}
	|>]
]