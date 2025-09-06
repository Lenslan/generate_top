module driver (in1, in2, out, en);
input wire [3-1:0] in1, in2;
output reg [3:0] out;
input en;
bufif0 ar[3:0] (out, in, en); // array of three-state buffers
endmodule

// module busdriver_equiv (busin, bushigh, buslow, enh, enl);
// input [15:0] busin;
// output reg [7:0] bushigh, buslow;
// input enh, enl;
// driver busar[3:0] (.out({bushigh, buslow}), .in(busin),
// .en({enh, enh, enl, enl}));
// endmodule
