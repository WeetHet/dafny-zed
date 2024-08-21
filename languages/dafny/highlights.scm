("method") @keyword.function
("function") @keyword.function
("predicate") @keyword.function

("while" @keyword.control.repeat)
("return" @keyword.control)
("if" @keyword.control)
("else" @keyword.control)

(["break" "continue"] @keyword.control)

("var" @keyword.storage.type)

(["requires" "ensures" "invariant"] @keyword.control.spec)
("assert" @keyword.control.spec)
(["forall"] @keyword.control.spec)
; (["exists"] @keyword.control.spec)

(expression ["==" "<=" ">=" "!=" ">" "<"] @operator)
(assignment ":=" @operator)

(comment) @comment
(identifier) @variable
(type) @type
