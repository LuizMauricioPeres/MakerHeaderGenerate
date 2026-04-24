/*
* Variable declarations example
*/

MEMVAR usuario, nSerial, cModulo

PROCEDURE DeclareVars()
   PUBLIC cor01, cor02, cor03
   PUBLIC lRecados := .t., nivel := 0
   PUBLIC aVars := {}
   
   STATIC nCounter := 0
   STATIC cCache := ''
   
RETURN

FUNCTION ProcessVars()
   LOCAL x, y, z
   x := ProcessItem()
   y := ValidateData( x )
   z := SaveResult( x, y )
RETURN z
