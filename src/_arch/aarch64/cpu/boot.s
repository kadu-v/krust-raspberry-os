//-------------------------------------------------------------------------------------------------
// Public Code
//-------------------------------------------------------------------------------------------------

.section .text._start // 新しいセクション .text._start を定義

//-------------------------------------------------------------------------------------------------
// fn _start()
//-------------------------------------------------------------------------------------------------
_start:
    // Infinitely wait for events
.L_parking_loop:
    wfe
    b   .L_parking_loop

.size   _start, . - _start // _start関数のサイズを計算する -> 最適化に用いられることがある？
.type   _start, function   // _startシンボルのタイプ(function, object)を設定 -> functionを設定
.global _start             // _startをグローバルなシンボルに設定 -> 上書きされることがない