BeginPackage["ConnorGray`Markdown`MarkdownToNotebook`"]

Begin["`Private`"]

Needs["Wolfram`ErrorTools`"]

Needs["ConnorGray`Markdown`"]

$nestedBlockStack := Raise[
	MarkdownError,
	"Invalid unscoped attempt to access $nestedBlockStack"
]

(*========================================================*)

SetFallthroughError[MarkdownToNotebook]

MarkdownToNotebook[markdown0_] := Handle[_Failure] @ WrapRaised[
	MarkdownError,
	"Error converting Markdown to notebook"
] @ Block[{
	(* A document is being processed, but we're at the document root so
		nothing is on the stack yet. *)
	$nestedBlockStack = {}
}, Module[{
	cells
},
	cells = {markdownToCells[markdown0]};

	RaiseAssert[MatchQ[cells, {___Cell}]];

	Notebook[
		cells,
		StyleDefinitions -> FrontEnd`FileName[
			{"ConnorGray"},
			"Markdown.nb",
			CharacterEncoding -> "UTF-8"
		]
	]
]]

(*========================================================*)

SetFallthroughError[markdownToCells]

(*
	markdownToCells[expr] converts a MarkdownElement or {___MarkdownElement}
	expression into notebook Cell content.

	The returned cells expression will always be in one of the following
	forms:

	* Cell[...]
	* Splice[{___Cell}]
	* Nothing
*)

markdownToCells[markdown0_] := ConfirmReplace[markdown0, {
	list_?ListQ :> (
		Splice @ Map[markdownToCells, list]
	),

	MarkdownElement["Heading", level_?IntegerQ, inlines_List] :> Module[{
		style
	},
		(* TID:240625/5: Heading not supported inside list item *)
		(* TODO: TID:240625/6: Heading not supported inside blockquote *)
		requireDocumentRoot[markdown0];

		style = ConfirmReplace[level, {
			1 -> "Title",
			(* TODO: This style isn't used often anymore. *)
			2 -> "Chapter",
			3 -> "Section",
			4 -> "Subsection",
			5 -> "Subsubsection",
			6 -> "Subsubsubsection",
			other_ :> Raise[
				MarkdownError,
				"Invalid heading level: ``",
				InputForm[other]
			]
		}];

		Cell[
			TextData[{inlinesToTextData[inlines]}],
			style
		]
	],

	element:MarkdownElement["Paragraph", inlines_List] :> WrapRaised[
		MarkdownError,
		"Error converting Paragraph element: ``",
		InputForm[element]
	] @ Module[{
		cellStyle = paragraphCellStyle[]
	},
		Cell[
			TextData[{inlinesToTextData[inlines]}],
			cellStyle
		]
	],

	MarkdownElement["CodeBlock", infoString_?StringQ, program_?StringQ] :> (
		Cell[
			program,
			"Program",
			TaggingRules -> <|
				"ConnorGray/Markdown" -> <|
					"CodeBlockInfoString" -> infoString
				|>
			|>
		]
	),

	(* TID:240625/3: Converting simple lists to cells *)
	element:MarkdownElement["List", listItems_List] :> Module[{},
		(* TID:240630/1: Error for expected List children to be ListItem. *)
		If[!MatchQ[listItems, {MarkdownElement["ListItem", {___}]..}],
			Raise[
				MarkdownError,
				<| "MarkdownElement" -> element |>,
				"Expected \"List\" element children to be \"ListItem\" elements."
			];
		];

		Splice @ Map[
			listItem |-> withBlockStack[
				"List",
				markdownToCells[listItem]
			],
			listItems
		]
	],

	MarkdownElement["ListItem", listItemBlocks_List] :> Module[{},
		(* Note:
			Record the index of the inner block in this list item
			we're inside of. This is used later to determine the cell style
			(Item vs ItemParagraph) used to indicate whether this cell "begins"
			a list item vs just continues it. *)
		(* TID:240625/4: Converting multi-block list items to cells *)
		Splice @ MapIndexed[
			{block, pos} |-> withBlockStack[
				{"ListItem", First[pos]},
				markdownToCells[block]
			],
			listItemBlocks
		]
	],

	(*================================*)
	(* Unrecognized Markdown form     *)
	(*================================*)

	other_ :> Raise[
		MarkdownError,
		<| "MarkdownExpression" -> other |>,
		"Unrecognized Markdown form"
	]
}]

(*====================================*)

SetFallthroughError[inlinesToTextData]

inlinesToTextData[inlines0_] := ConfirmReplace[inlines0, {
	list_?ListQ :> Splice @ Map[inlinesToTextData, list],

	MarkdownElement["Text", text_?StringQ] :> text,

	(* TID:240625/2: Converting softbreaks to cells *)
	MarkdownElement["SoftBreak"] :> "\n",

	(*-------------------------------------------------*)
	(* TID:240625/1: Converting inline styles to cells *)
	(*-------------------------------------------------*)

	MarkdownElement["Code", text_?StringQ] :> StyleBox[text, "Code"],

	MarkdownElement["Strong", strongInlines_List] :> (
		StyleBox[
			{Splice @ Map[inlinesToTextData, strongInlines]},
			FontWeight -> "Bold"
		]
	),

	MarkdownElement["Emphasis", emphasisInlines_List] :> (
		StyleBox[
			{Splice @ Map[inlinesToTextData, emphasisInlines]},
			FontSlant -> "Italic"
		]
	),

	(*--------------------------------*)
	(* Error for unrecognized forms   *)
	(*--------------------------------*)

	other_ :> Raise[
		MarkdownError,
		<| "Expression" -> other |>,
		"Not a recognized Markdown inline element."
	]
}]

(*====================================*)

SetFallthroughError[paragraphCellStyle]

paragraphCellStyle[] := ConfirmReplace[$nestedBlockStack, {
	(* If this Paragraph element is at the root of the document, its a "Text"
		cell. *)
	{} -> "Text",

	(*	If this is the *first* inner block in a list item,
		return one of the Item styles that has a bullet dingbat marker,
		based on the list depth. *)
	{___, {"ListItem", 1}} :> ConfirmReplace[
		Count[$nestedBlockStack, "List"],
		{
			1 -> "Item",
			2 -> "Subitem",
			3 -> "Subsubitem",
			(* TID:240627/1: Error for over-nested list item. *)
			other_ :> Raise[
				MarkdownError,
				"Unsupported nested list depth beyond 3: ``",
				InputForm[other]
			]
		}
	],

	(* 	If this is one of the "rest" / "trailing" inner blocks (i.e. not the
		first) in a list item, use one of the ItemParagraph styles that is
		indented but does not have its own bullet dingbat marker. *)
	{___, {"ListItem", _?IntegerQ}} :> ConfirmReplace[
		Count[$nestedBlockStack, "List"],
		{
			1 -> "ItemParagraph",
			2 -> "SubitemParagraph",
			3 -> "SubsubitemParagraph",
			other_ :> Raise[
				MarkdownError,
				"Unsupported nested list depth: ``",
				InputForm[other]
			]
		}
	],

	(* TODO: Test for this error? *)
	other_ :> Raise[
		MarkdownError,
		<|
			"MarkdownElement" -> element,
			"CurrentBlockStack" -> $nestedBlockStack
		|>,
		"Unsupported scope for Markdown element to appear"
	]
}]

(*========================================================*)
(* Helpers                                                *)
(*========================================================*)

SetFallthroughError[requireDocumentRoot]

requireDocumentRoot[
	element:MarkdownElement[type_?StringQ, ___]
] := (
	If[$nestedBlockStack =!= {},
		Raise[
			MarkdownError,
			<|
				"MarkdownElement" -> element,
				"CurrentBlockStack" -> $nestedBlockStack
			|>,
			"Unsupported `` type Markdown element not at document root",
			InputForm[type]
		]
	];
)

(*====================================*)

SetFallthroughError[withBlockStack]

Attributes[withBlockStack] = {HoldRest}

withBlockStack[
	blockSpec_,
	heldExpr_
] := Block[{
	$nestedBlockStack = Append[$nestedBlockStack, blockSpec]
},
	(* Release held argument *)
	heldExpr
]

(*========================================================*)

End[]

EndPackage[]