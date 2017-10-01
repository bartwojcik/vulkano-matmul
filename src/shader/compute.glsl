#version 450

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

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
};

void main() {
    uint idx = gl_WorkGroupID.x;
    uint idy = gl_WorkGroupID.y;
    // n - matrix_b's number of columns
    uint n = gl_NumWorkGroups.x;
    float result = 0.0;
    for (uint i = 0; i < k; ++i) {
        result += matrix_a.data[idy * k + i] * matrix_b.data[idx + i * n];
    }
    matrix_out.data[idy * n + idx] = result;
}