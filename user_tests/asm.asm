	.data
msg: word 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21, 0x0
i_main: word 0x0
__tmp_val: word 0x0
__tmp_0: word 0x0
__tmp_1: word 0x0
__tmp_2: word 0x0
__tmp_3: word 0x0
__tmp_4: word 0x0
__tmp_5: word 0x0
__tmp_6: word 0x0
__tmp_7: word 0x0
__tmp_8: word 0x0
__tmp_9: word 0x0
__tmp_10: word 0x0
__tmp_11: word 0x0
__tmp_12: word 0x0
__tmp_13: word 0x0
__tmp_14: word 0x0
__tmp_15: word 0x0
__tmp_16: word 0x0
__tmp_17: word 0x0
__tmp_18: word 0x0
__tmp_19: word 0x0
__tmp_20: word 0x0
__tmp_21: word 0x0
__tmp_22: word 0x0
__tmp_23: word 0x0
__tmp_24: word 0x0
__tmp_25: word 0x0
__tmp_26: word 0x0
__tmp_27: word 0x0
__tmp_28: word 0x0
__tmp_29: word 0x0
__tmp_30: word 0x0
__tmp_31: word 0x0
__tmp_idx_0: word 0x0
__tmp_idx_1: word 0x0
__tmp_idx_2: word 0x0
__tmp_idx_3: word 0x0
__tmp_idx_4: word 0x0
__tmp_idx_5: word 0x0
__tmp_idx_6: word 0x0
__tmp_idx_7: word 0x0
__tmp_ptr_0: word 0x0
__tmp_ptr_1: word 0x0
__tmp_ptr_2: word 0x0
__tmp_ptr_3: word 0x0
__tmp_ptr_4: word 0x0
__tmp_ptr_5: word 0x0
__tmp_ptr_6: word 0x0
__tmp_ptr_7: word 0x0

	.code
main:
	LD #0
	ST i_main
.L0:
	LD #0
	ST __tmp_0
	LD i_main
	ST __tmp_idx_1
	LD #0
	ADD __tmp_idx_1
	ST __tmp_ptr_1
	LD [__tmp_ptr_1]
	CMP __tmp_0
	JZS .L2
	LD #1
	JMP .L3
.L2:
	LD #0
.L3:
	JZS .L1
	LD i_main
	ST __tmp_idx_0
	LD #0
	ADD __tmp_idx_0
	ST __tmp_ptr_0
	LD [__tmp_ptr_0]
	ST 0x1
	LD #1
	ST __tmp_0
	LD i_main
	ADD __tmp_0
	ST i_main
	JMP .L0
.L1:
	HALT
