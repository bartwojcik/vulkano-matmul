#version 450

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly buffer MatA {
    float data[];
} matrix_a;

layout(set = 0, binding = 1) readonly buffer MatB {
    float data[];
} matrix_b;

layout(set = 0, binding = 2) writeonly buffer MatOut {
    float data[];
} matrix_out;

layout(push_constant) uniform PushConstants {
    uint k; // matrix_a's number of columns and matrix_b's number of rows
    uint m; // matrix_c's number of rows
    uint n; // matrix_c's number of columns
};

void main() {
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;

    float result = 0.0;
    for (uint i = 0; i < k; ++i) {
        result += matrix_a.data[y * k + i] * matrix_b.data[x + i * n];
    }
    matrix_out.data[y * n + x] = result;
}