/*
 * Copyright (C) 2021-2023 Synopsys, Inc. All rights reserved.
 *
 * SYNOPSYS CONFIDENTIAL - This is an unpublished, confidential, and
 * proprietary work of Synopsys, Inc., and may be subject to patent,
 * copyright, trade secret, and other legal or contractual protection.
 * This work may be used only pursuant to the terms and conditions of a
 * written license agreement with Synopsys, Inc. All other use, reproduction,
 * distribution, or disclosure of this work is strictly prohibited.
 *
 * The entire notice above must be reproduced on all authorized copies.
 */

// `include "npu_macros.svh"
// `include "npu_defines.v"
module npu_afifo_r
  #(
    parameter int FIFO_DATA_WIDTH = 8,
    parameter int FIFO_SIZEL2     = 2
  )
  (
   input  logic                        read_clk,
   input  logic                        read_rst,
   input  logic                        read_soft_rst,
   output logic                        read_valid,
   input  logic                        read_accept,
   output logic [FIFO_DATA_WIDTH-1:0]  read_data,
  //  output logic [`NUM_FLANES(FIFO_DATA_WIDTH)-1:0][(1<<FIFO_SIZEL2)-1:0] rdpnt,
   input  logic [FIFO_DATA_WIDTH-1:0]  rdata,
   input  logic [FIFO_SIZEL2:0]        wpnt_a,
   output logic [FIFO_SIZEL2:0]        rpnt
   );
  

endmodule : npu_afifo_r
