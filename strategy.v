

module top(price,threshold,volatility,buy_signal,sell_signal);
    
    // Module arguments
    input wire  [31:0] price;
    input wire  [31:0] threshold;
    input wire  [31:0] volatility;
    output reg  buy_signal;
    output reg  sell_signal;
    
    // Update code
    always @(*) begin
        if (price + volatility < threshold) begin
            buy_signal = 1'b1;
            sell_signal = 1'b0;
        end
        else if (price > threshold + volatility) begin
            buy_signal = 1'b0;
            sell_signal = 1'b1;
        end
        else begin
            buy_signal = 1'b0;
            sell_signal = 1'b0;
        end
    end
    
endmodule // top
