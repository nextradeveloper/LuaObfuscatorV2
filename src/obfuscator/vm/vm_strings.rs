pub static VARIABLE_DECLARATION: &str = "
local String = string
local StringChar = String.char
local StringByte = String.byte
local StringSub = String.sub
local StringReverse = String.reverse
local StringFindReal = String.find
-- I had to do this BS because lua returns start and end index and I didn't want to deal with that
local StringFind = function(str, val) local a, _ = StringFindReal(str, val) return a - 1 end
local StringConcat = function(...)
	local str = ''
	local strs = {...}
	for i = 1, #strs do
		str = str .. strs[i]
	end

	return str
end
local Select = select
local Table = table
local Math = math
local Error = error
local Pairs = pairs
local IPairs = ipairs
local TableConcat = Table.concat
local TableInsert = Table.insert
local function TableUnpack(tbl, i, j)
    i = i or 1
    j = j or #tbl
    if j-i+1 >= 10 then
        return tbl[i], tbl[i + 1], tbl[i + 2], tbl[i + 3], tbl[i + 4],
               tbl[i + 5], tbl[i + 6], tbl[i + 7], tbl[i + 8], tbl[i + 9],
               TableUnpack(tbl, i + 10, j)
    end
    if i <= j then return tbl[i], TableUnpack(tbl, i + 1, j) end
end
local TableCreate = function(len)
	return {TableUnpack({}, 1, len or 1)}
end
local TablePack = function(...)
	return { n = Select(StringChar(35), ...), ... }
end
local TableMove = function(src, first, last, offset, dst)
	for i = 0, last - first do
		dst[offset + i] = src[first + i]
	end
end
local TableMerge = function(...)
	local newTable = {}
	local tbls = {...}
	for i = 1, #tbls do
		for j = 1, #(tbls[i]) do
			TableInsert(newTable, tbls[i][j])
		end
	end

	return newTable
end
local Getfenv = getfenv
local MathFloor = Math.floor
local MathMax = Math.max
local Pcall = pcall
local MathAbs = Math.abs
local Tonumber = tonumber

local RangeGen = function(inputStart, finish, step)
	step = step or 1
	local start = finish and inputStart or 1
	finish = finish or inputStart

	local a = {}

	for i = start, finish, step do
		TableInsert(a, i)
	end

	return a
end

local getBitwise = (function()
	local function tobittable_r(x, ...)
		if (x or 0) == 0 then
			return ...
		end
		return tobittable_r(MathFloor(x / 2), x % 2, ...)
	end

	local function tobittable(x)
		if x == 0 then
			return { 0 }
		end
		return { tobittable_r(x) }
	end

	local function makeop(cond)
		local function oper(x, y, ...)
			if not y then
				return x
			end
			x, y = tobittable(x), tobittable(y)
			local xl, yl = #x, #y
			local t, tl = {}, MathMax(xl, yl)
			for i = 0, tl - 1 do
				local b1, b2 = x[xl - i], y[yl - i]
				if not (b1 or b2) then
					break
				end
				t[tl - i] = (cond((b1 or 0) ~= 0, (b2 or 0) ~= 0) and 1 or 0)
			end
			return oper(Tonumber(TableConcat(t), 2), ...)
		end
		return oper
	end

	---
	-- Perform bitwise AND of several numbers.
	-- Truth table:
	--   band(0,0) -> 0,
	--   band(0,1) -> 0,
	--   band(1,0) -> 0,
	--   band(1,1) -> 1.
	-- @class function
	-- @name band
	-- @param ...  Numbers.
	-- @return  A number.
	local band = makeop(function(a, b)
		return a and b
	end)

	---
	-- Shift a number's bits to the left.
	-- Roughly equivalent to (x * (2^bits)).
	-- @param x  The number to shift (number).
	-- @param bits  Number of positions to shift by (number).
	-- @return  A number.
	local function blshift(x, bits)
		return MathFloor(x) * (2 ^ bits)
	end

	---
	-- Shift a number's bits to the right.
	-- Roughly equivalent to (x / (2^bits)).
	-- @param x  The number to shift (number).
	-- @param bits  Number of positions to shift by (number).
	-- @return  A number.
	local function brshift(x, bits)
		return MathFloor(MathFloor(x) / (2 ^ bits))
	end

	return band, brshift, blshift
end)
local BitAnd, BitRShift, BitLShift = getBitwise()
";

pub static DESERIALIZER: &str = "
local lua_bc_to_state
local lua_wrap_state
local stm_lua_func

-- int rd_int_basic(string src, int s, int e, int d)
-- @src - Source binary string
-- @s - Start index of a little endian integer
-- @e - End index of the integer
-- @d - Direction of the loop
local function rd_int_basic(src, s, e, d)
	local num = 0

	-- if bb[l] > 127 then -- signed negative
	-- 	num = num - 256 ^ l
	-- 	bb[l] = bb[l] - 128
	-- end

	for i = s, e, d do
		local mul = 256 ^ MathAbs(i - s)

		num = num + mul * StringByte(src, i, i)
	end

	return num
end

-- double rd_dbl_basic(byte f1..8)
-- @f1..8 - The 8 bytes composing a little endian double
local function rd_dbl_basic(f1, f2, f3, f4, f5, f6, f7, f8)
	local sign = (-1) ^ BitRShift(f8, 7)
	local exp = BitLShift(BitAnd(f8, 0x7F), 4) + BitRShift(f7, 4)
	local frac = BitAnd(f7, 0x0F) * 2 ^ 48
	local normal = 1

	frac = frac + (f6 * 2 ^ 40) + (f5 * 2 ^ 32) + (f4 * 2 ^ 24) + (f3 * 2 ^ 16) + (f2 * 2 ^ 8) + f1 -- help

	if exp == 0 then
		if frac == 0 then
			return sign * 0
		else
			normal = 0
			exp = 1
		end
	elseif exp == 0x7FF then
		if frac == 0 then
			return sign * (1 / 0)
		else
			return sign * (0 / 0)
		end
	end

	return sign * 2 ^ (exp - 1023) * (normal + frac / 2 ^ 52)
end

-- int rd_int_le(string src, int s, int e)
-- @src - Source binary string
-- @s - Start index of a little endian integer
-- @e - End index of the integer
local function rd_int_le(src, s, e) return rd_int_basic(src, s, e - 1, 1) end

-- double rd_dbl_le(string src, int s)
-- @src - Source binary string
-- @s - Start index of little endian double
local function rd_dbl_le(src, s) return rd_dbl_basic(StringByte(src, s, s + 7)) end

-- byte stm_byte(Stream S)
-- @S - Stream object to read from
local function stm_byte(S)
	local idx = S[1]
	local bt = StringByte(S[2], idx, idx)

	S[1] = idx + 1
	return bt
end

-- string stm_string(Stream S, int len)
-- @S - Stream object to read from
-- @len - Length of string being read
local function stm_string(S, len)
	local pos = S[1] + len
	local str = StringSub(S[2], S[1], pos - 1)

	S[1] = pos
	return str
end

local function stm_int16(S)
	local pos = S[1] + 2
	local int = rd_int_le(S[2], S[1], pos)
	S[1] = pos

	return int
end

local function stm_int32(S)
	local pos = S[1] + 4
	local int = rd_int_le(S[2], S[1], pos)
	S[1] = pos

	return int
end

local function stm_int64(S)
	local pos = S[1] + 8
	local int = rd_int_le(S[2], S[1], pos)
	S[1] = pos

	return int
end

local function stm_num(S)
	local flt = rd_dbl_le(S[2], S[1])
	S[1] = S[1] + 8

	return flt
end

-- string stm_lstring(Stream S)
-- @S - Stream object to read from
local function stm_lstring(S)
	local len = stm_int64(S)
	local str

	if len ~= 0 then str = StringSub(stm_string(S, len), 1, -2) end

	return str
end

local function stm_inst_list(S)
	local len = stm_int64(S)
	local list = TableCreate(len)

	for i = 1, len do
		local ins = stm_int16(S)
		local op = BitAnd(BitRShift(ins, 4), 0x3f)
		local args = BitAnd(BitRShift(ins, 2), 3)
		local isConstantB = BitAnd(BitRShift(ins, 1), 1) == 1
		local isConstantC = BitAnd(ins, 1) == 1
		local data = {}
		data[$OPCODE$] = op
		data[$A_REGISTER$] = stm_byte(S)

		if args == 1 then -- ABC
			data[$B_REGISTER$] = stm_int16(S)
			data[$C_REGISTER$] = stm_int16(S)
			data[$IS_KB$] = isConstantB and data[$B_REGISTER$] > 0xFF -- post process optimization
			data[$IS_KC$] = isConstantC and data[$C_REGISTER$] > 0xFF
		elseif args == 2 then -- ABx
			data[$B_REGISTER$] = stm_int32(S)
			data[$IS_CONST$] = isConstantB
		elseif args == 3 then -- AsBx
			data[$B_REGISTER$] = stm_int32(S) - 131071
		end

		list[i] = data
	end


	return list
end

local function stm_sub_list(S, src)
	local len = stm_int64(S)
	local list = TableCreate(len)

	for i = 1, len do
		list[i] = stm_lua_func(S, src) -- offset +1 in CLOSURE
	end

	return list
end
";

pub static DESERIALIZER_2: &str = "
function stm_lua_func(stream, psrc)
	local src = stm_lstring(stream) or psrc -- source is propagated

	local proto = {}
	proto[$SOURCE_NAME$] = src

	-- stream:s_int() -- line defined
	-- stream:s_int() -- last line defined

	proto[$UPVALUE_COUNT$] = stm_byte(stream) -- num upvalues
	proto[$PARAMETER_COUNT$] = stm_byte(stream) -- num params


	-- stm_byte(stream) -- vararg flag
	-- proto.max_stack = stm_byte(stream) -- max stack size
";

pub static DESERIALIZER_3: &str = "
-- post process optimization
for _, v in IPairs(proto[$OPCODE_LIST$]) do
	if v[$IS_CONST$] then
		v[$CONSTANT$] = proto[$CONSTANT_LIST$][v[$B_REGISTER$] + 1] -- offset for 1 based index
	else
		if v[$IS_KB$] then v[$CONST_B$] = proto[$CONSTANT_LIST$][v[$B_REGISTER$] - 0xFF] end

		if v[$IS_KC$] then v[$CONST_C$] = proto[$CONSTANT_LIST$][v[$C_REGISTER$] - 0xFF] end
	end
end

return proto
end

function lua_bc_to_state(src)
-- stream object
local stream = {
	-- data
	1,
	src
}

return stm_lua_func(stream, '')
end
";

pub static RUN_HELPERS: &str = "
local function close_lua_upvalues(list, index)
	for i, uv in Pairs(list) do
		if uv[1] >= index then
			-- Replace with indexes if uncommenting
			--uv.value = uv.store[uv.index] -- store value
			--uv.store = uv
			--uv.index = 'value' -- self reference
			list[i] = nil
		end
	end
end

local function open_lua_upvalue(list, index, memory)
	local prev = list[index]

	if not prev then
		prev = {index, memory}
		list[index] = prev
	end

	return prev
end

local function on_lua_error(failed, err)
	local src = failed[2]
	-- local line = failed.lines[failed.pc - 1]
	local line = 0

	Error(StringConcat(src, ':', line, ':', err), 0)
end
";

pub static RUN_HELPERS_LI: &str = "
local function close_lua_upvalues(list, index)
	for i, uv in Pairs(list) do
		if uv[1] >= index then
			--uv.value = uv.store[uv.index] -- store value
			--uv.store = uv
			--uv.index = 'value' -- self reference
			list[i] = nil
		end
	end
end

local function open_lua_upvalue(list, index, memory)
	local prev = list[index]

	if not prev then
		prev = {index, memory}
		list[index] = prev
	end

	return prev
end

local function on_lua_error(failed, err)
	local src = failed[2]
	local line = failed[3][failed[1] - 1]

	Error(StringConcat(src, ':', line, ':', err), 0)
end
";

pub static RUN: &str = "
local function run_lua_func(state, env, upvals)
	local code = state[3]
	local subs = state[4]
	local vararg = state[1]

	local top_index = -1
	local open_list = {}
	local memory = state[2]
	local pc = state[5]

	local function constantB(inst)
		return inst[$IS_KB$] and inst[$CONST_B$] or memory[inst[$B_REGISTER$]]
	end

	local function constantC(inst)
		return inst[$IS_KC$] and inst[$CONST_C$] or memory[inst[$C_REGISTER$]]
	end

	local function safe_arith(a, b, op)
		-- FiveM compatibility: treat nil values as 0 for arithmetic operations
		-- This is common in game scripting environments where undefined values default to 0
		a = a or 0
		b = b or 0
		return op(a, b)
	end

	while true do
		local inst = code[pc]
		local op = inst[$OPCODE$]
		pc = pc + 1

";

pub static RUN_2: &str = "
	state[5] = pc
	end
end

function lua_wrap_state(proto, env, upval)
	env = env or Getfenv(0)

	local function wrapped(...)
		local passed = TablePack(...)
		local memory = TableCreate()
		local vararg = {0, {}}

		TableMove(passed, 1, proto[$PARAMETER_COUNT$], 0, memory)

		if proto[$PARAMETER_COUNT$] < passed.n then
			local start = proto[$PARAMETER_COUNT$] + 1
			local len = passed.n - proto[$PARAMETER_COUNT$]

			vararg[1] = len
			TableMove(passed, start, start + len - 1, 1, vararg[2])
		end

		local state = {vararg, memory, proto[$OPCODE_LIST$], proto[$PROTO_LIST$], 1}

		local result = TablePack(Pcall(run_lua_func, state, env, upval))

		if result[1] then
			return TableUnpack(result, 2, result.n)
		else
			local failed = {state[5], proto[$SOURCE_NAME$] --[[,lines = proto.lines]]}

			on_lua_error(failed, result[2])

			return
		end
	end

	return wrapped
end
";

pub static RUN_2_LI: &str = "
	state[5] = pc
	end
end

function lua_wrap_state(proto, env, upval)
	env = env or Getfenv(0)

	local function wrapped(...)
		local passed = TablePack(...)
		local memory = TableCreate()
		local vararg = {0, {}}

		TableMove(passed, 1, proto[$PARAMETER_COUNT$], 0, memory)

		if proto[$PARAMETER_COUNT$] < passed.n then
			local start = proto[$PARAMETER_COUNT$] + 1
			local len = passed.n - proto[$PARAMETER_COUNT$]

			vararg[1] = len
			TableMove(passed, start, start + len - 1, 1, vararg[2])
		end

		local state = {vararg, memory, proto[$OPCODE_LIST$], proto[$PROTO_LIST$], 1}

		local result = TablePack(Pcall(run_lua_func, state, env, upval))

		if result[1] then
			return TableUnpack(result, 2, result.n)
		else
			local failed = {state[5], proto[$SOURCE_NAME$], proto[$LINE_LIST$]}

			on_lua_error(failed, result[2])

			return
		end
	end

	return wrapped
end
";
