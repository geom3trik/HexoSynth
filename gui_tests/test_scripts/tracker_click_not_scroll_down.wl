# Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
# This file is a part of HexoSynth. Released under GPL-3.0-or-later.
# See README.md and COPYING for details.

!@import t = wlambda_lib:test_lib;
!@import hx;

hx:query_state[];

$["click_no_scroll" => {
    # Create a tracker and select it
    hx:set_cell $i(0,0) ${ node_id = "tseq" => 0 };
    t:move_to_hex $i(0, 0);
    t:mouse_click :left;

    !(first_line_x, first_line_y) = (hx:id_by_text "00").0.1;

    hx:mouse_move
        first_line_x + 30
        first_line_y - 10;

    t:mouse_click :left;

    t:key :Enter;   # Pattern edit mode.
    t:key :f :f :f; # Enter "fff" into the first cell.

    hx:query_state[];

    std:assert
        is_none[hx:id_by_text["255"]]
        "tracker scrolled down despite click on the header";

    std:assert_eq
        hx:id_by_text["00"].0.0.1
        "DBGID_PATEDIT_ROW"
        "first tracker row still visible";

    std:assert_eq
        (hx:id_by_text :fff).0.0.1
        "DBGID_PATEDIT_CELL";

    hx:query_state[];
    hx:query_state[];
    hx:query_state[];
    hx:query_state[];
    !pat = hx:pattern_data_for_tracker 0;
    std:assert_eq pat.get_cursor[]   $i(2, 2)  "cursor advanced";
    std:assert_eq (pat.get_cell 0 0) "fff"     "first cell contains right data";
}]
