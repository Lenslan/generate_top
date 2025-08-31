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

`include "npu_macros.svh"
`include "npu_defines.v"
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
   output logic [`NUM_FLANES(FIFO_DATA_WIDTH)-1:0][(1<<FIFO_SIZEL2)-1:0] rdpnt,
   input  logic [FIFO_DATA_WIDTH-1:0]  rdata,
   input  logic [FIFO_SIZEL2:0]        wpnt_a
   output logic [FIFO_SIZEL2:0]        rpnt
   );
  localparam int FIFO_SIZE = 1<<FIFO_SIZEL2;
// spyglass disable_block SelfDeterminedExpr-ML
// SMD: Self-determin expression
// SJ: Reviewed
  // gray to binary and vv conversion functions
  function automatic logic [FIFO_SIZEL2:0] bin_to_gray(input logic [FIFO_SIZEL2:0] bin_value);
    begin : gray_to_bin_body
      int i;
      bin_to_gray = 0;
      bin_to_gray[FIFO_SIZEL2] = bin_value[FIFO_SIZEL2];
      for (i = FIFO_SIZEL2-1; i >= 0; i--)
        bin_to_gray[i] = bin_value[i + 1] ^ bin_value[i];
    end : gray_to_bin_body
  endfunction : bin_to_gray
  function automatic logic [FIFO_SIZEL2:0] gray_to_bin(input logic [FIFO_SIZEL2:0] gray_value);
    begin : bin_to_gray_body
      int i;
      gray_to_bin = 0;
      gray_to_bin[FIFO_SIZEL2] = gray_value[FIFO_SIZEL2];
      for (i = FIFO_SIZEL2-1; i >= 0; i--)
        gray_to_bin[i] = gray_to_bin[i + 1] ^ gray_value[i];
    end : bin_to_gray_body
  endfunction : gray_to_bin
// spyglass enable_block SelfDeterminedExpr-ML


  // local wires
  logic                       en;
  logic [FIFO_SIZEL2:0]       rd_pnt_r;
  logic [FIFO_SIZEL2:0]       rd_shadow_r;
  logic [FIFO_SIZE-1:0]       rd_addr_r;
  logic [FIFO_SIZEL2:0]       rd_pnt_nxt;
  logic [FIFO_SIZEL2:0]       rd_shadow_nxt;
  logic [FIFO_SIZE-1:0]       rd_addr_nxt;
  logic [FIFO_SIZEL2:0]       wr_pnt_meta_r;
  logic [FIFO_SIZEL2:0]       wr_pnt_sync_r;
  logic                       readi_valid;
  logic                       readi_accept;
  logic [FIFO_DATA_WIDTH-1:0] readi_data;
  logic [FIFO_DATA_WIDTH-1:0] read_data_r;
  logic                       read_valid_nxt;
  logic                       read_valid_r;
  logic [FIFO_DATA_WIDTH-1:0] data_rst_val;

  // assignments
  assign en             = readi_valid & readi_accept | read_soft_rst;
  assign rdpnt          = {(`NUM_FLANES(FIFO_DATA_WIDTH)){rd_addr_r}};
  assign rpnt           = rd_shadow_r;
  assign readi_valid    = wr_pnt_sync_r != rd_pnt_r;
  assign rd_pnt_nxt     = read_soft_rst ? '0                     : bin_to_gray(gray_to_bin(rd_pnt_r)+unsigned'(1));
  assign rd_shadow_nxt  = read_soft_rst ? bin_to_gray(FIFO_SIZE) : bin_to_gray(gray_to_bin(rd_shadow_r)+unsigned'(1));
// spyglass disable_block W164a
// SMD:Width mismatch
// SJ :Ignore $unsigned(i) width, MSB trunk
  assign rd_addr_nxt    = read_soft_rst ? unsigned'(1)           : { rd_addr_r, rd_addr_r[FIFO_SIZE-1] };
// spyglass enable_block W164a
  assign read_valid_nxt = (~read_soft_rst) & ((read_valid_r & ~read_accept) | readi_valid);
  assign readi_accept   = read_accept | (~read_valid_r);
  assign read_valid     = read_valid_r;
  assign read_data      = read_data_r;

  assign data_rst_val = '0;

  // read state
  // outputs: wr_pnt_meta_r, wr_pnt_sync_r, rd_pnt_r, rd_shadow_r, rd_addr_r, read_valid_r, read_data_r
  always_ff @(posedge read_clk or
              posedge read_rst)
  begin : rd_proc
    if (read_rst == 1'b1)
    begin
      wr_pnt_meta_r <= '0;
      wr_pnt_sync_r <= '0;
      rd_pnt_r      <= '0;
      rd_shadow_r   <= bin_to_gray(FIFO_SIZE);
      rd_addr_r     <= unsigned'(1);
      read_valid_r  <= 1'b0;
      read_data_r   <= data_rst_val;
    end
    else 
    begin
// spyglass disable_block Ac_unsync02
// SMD: Unsync Crossing
// SJ: Reviewed and Waived, Use JIRA to Track
      wr_pnt_meta_r <= read_soft_rst ? '0 : wpnt_a;
      wr_pnt_sync_r <= read_soft_rst ? '0 : wr_pnt_meta_r;
// spyglass enable_block Ac_unsync02
      read_valid_r  <= read_valid_nxt;
      if (en) begin
        rd_pnt_r    <= rd_pnt_nxt;
        rd_shadow_r <= rd_shadow_nxt;
        rd_addr_r   <= rd_addr_nxt;
      end
      if (readi_valid & readi_accept) begin
// spyglass disable_block Ac_unsync02
// SMD: Unsync Crossing
// SJ: Reviewed and Waived, Use JIRA to Track
        read_data_r <= rdata;
// spyglass enable_block Ac_unsync02
      end
    end
  end : rd_proc
  

endmodule : npu_afifo_r
