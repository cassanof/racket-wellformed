#reader(lib "htdp-beginner-reader.ss" "lang")((modname p2-code) (read-case-sensitive #t) (teachpacks ()) (htdp-settings #(#t constructor repeating-decimal #f #t none #f () #f)))

;; this is a test file

(+ 1 4 5) ; comment after


#| block comment!

here it ends |#


(define (my-func x)
  (+ x 5))

#;(1 2 4 5 6) ;whole sexpr comment


(+ (my-func 4) #;(my-func 3) 9) ; sexpr comment in sexpr


