Needs["ConnorGray`Markdown`"]

VerificationTest[
	ConvertToMarkdownElement @ Notebook[{
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
	ConvertToMarkdownElement @ Notebook[{
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

VerificationTest[
	ConvertToMarkdownElement @ Notebook[{
		Cell["Example Document", "Title"],
		Cell["This is some text", "Text"],
		Cell[
			BoxData @ RowBox[{
				RowBox[{"foo", "[", "x_", "]"}],
				":=",
				RowBox[{"x", "+", "1"}]
			}],
			"Input"
		],
		Cell[
			TextData[{
				"This is ",
				StyleBox["some ", FontWeight -> "Bold"],
				StyleBox["styled", FontWeight -> "Bold", FontSlant -> "Italic"],
				StyleBox[" text", FontWeight -> "Bold"],
				"."
			}],
			"Text"
		]
	}],
	{
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
	}
]

(*====================================*)
(* Conversion of Input cells          *)
(*====================================*)

VerificationTest[
	ConvertToMarkdownElement @ Cell[
		BoxData @ RowBox[{"2", "+", "2"}],
		"Input"
	],
	MarkdownElement["CodeBlock", "wolfram,cell:Input", "2 + 2"]
]

VerificationTest[
	ConvertToMarkdownElement @ Cell[
		BoxData @ RowBox[{
			RowBox[{"foo", "[", "x_", "]"}],
			":=",
			RowBox[{"x", "+", "1"}]
		}],
		"Input"
	],
	MarkdownElement["CodeBlock", "wolfram,cell:Input", "foo[x_] := x + 1"]
]