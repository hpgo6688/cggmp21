#ifndef DKG_NODE_FFI_H
#define DKG_NODE_FFI_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque pointer types
typedef struct DkgState DkgState;
typedef struct DkgResult DkgResult;

// Result structure
typedef struct {
    bool success;
    char* error_message;
    char* public_key;
    uint16_t node_index;
    size_t total_nodes;
} DkgResultStruct;

// Core DKG functions
DkgState* dkg_node_create(void);
void dkg_node_destroy(DkgState* state);

// Main DKG session function
DkgResult* dkg_node_run_session(
    uint16_t node_id,
    const char* relay_url,
    size_t total_nodes,
    const char* session_id
);

// Result accessor functions
void dkg_result_destroy(DkgResult* result);
bool dkg_result_get_success(const DkgResult* result);
const char* dkg_result_get_error_message(const DkgResult* result);
const char* dkg_result_get_public_key(const DkgResult* result);
uint16_t dkg_result_get_node_index(const DkgResult* result);
size_t dkg_result_get_total_nodes(const DkgResult* result);

#ifdef __cplusplus
}
#endif

#endif // DKG_NODE_FFI_H 