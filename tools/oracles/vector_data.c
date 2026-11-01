/* SPDX-License-Identifier: AGPL-3.0-or-later */
typedef struct { char kind; uint32_t bits[3]; } Operand;

static VariableValue operand(Operand input)
{
    VariableValue result = {0};
    if (input.kind == 'V') {
        result.type = VAR_VECTOR;
        float *storage = Scr_AllocVectorInternal();
        memcpy(storage, input.bits, 3 * sizeof(float));
        result.u.vectorValue = storage;
    } else {
        if (input.kind != 'I' && input.kind != 'F') abort();
        result.type = input.kind == 'I' ? VAR_INTEGER : VAR_FLOAT;
        memcpy(&result.u, &input.bits[0], sizeof(uint32_t));
    }
    return result;
}

static void print_operand(Operand input)
{
    printf(" %c %08" PRIx32 " %08" PRIx32 " %08" PRIx32,
           input.kind, input.bits[0], input.bits[1], input.bits[2]);
}

static void print_vector(const float *value, int arithmetic)
{
    for (unsigned int i = 0; i < 3; ++i) {
        uint32_t bits;
        memcpy(&bits, &value[i], sizeof(bits));
        if (arithmetic && isnan(value[i])) printf(" nan");
        else printf(" %08" PRIx32, bits);
    }
}

