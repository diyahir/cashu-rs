#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
typedef struct _Dart_Handle* Dart_Handle;

typedef struct DartCObject DartCObject;

typedef int64_t DartPort;

typedef bool (*DartPostCObjectFnType)(DartPort port_id, void *message);

typedef struct wire_uint_8_list {
  uint8_t *ptr;
  int32_t len;
} wire_uint_8_list;

typedef struct DartCObject *WireSyncReturn;

void store_dart_post_cobject(DartPostCObjectFnType ptr);

Dart_Handle get_dart_object(uintptr_t ptr);

void drop_dart_object(uintptr_t ptr);

uintptr_t new_dart_opaque(Dart_Handle handle);

intptr_t init_frb_dart_api_dl(void *obj);

void wire_init_cashu(int64_t port_);

void wire_get_balance(int64_t port_);

void wire_mint_tokens(int64_t port_, uint64_t amount, struct wire_uint_8_list *hash);

void wire_get_mint_payment_request(int64_t port_, uint64_t amount);

void wire_pay_invoice(int64_t port_, struct wire_uint_8_list *invoice);

void wire_import_token(int64_t port_, struct wire_uint_8_list *token);

void wire_join_federation(int64_t port_, struct wire_uint_8_list *federation);

struct wire_uint_8_list *new_uint_8_list_0(int32_t len);

void free_WireSyncReturn(WireSyncReturn ptr);

static int64_t dummy_method_to_enforce_bundling(void) {
    int64_t dummy_var = 0;
    dummy_var ^= ((int64_t) (void*) wire_init_cashu);
    dummy_var ^= ((int64_t) (void*) wire_get_balance);
    dummy_var ^= ((int64_t) (void*) wire_mint_tokens);
    dummy_var ^= ((int64_t) (void*) wire_get_mint_payment_request);
    dummy_var ^= ((int64_t) (void*) wire_pay_invoice);
    dummy_var ^= ((int64_t) (void*) wire_import_token);
    dummy_var ^= ((int64_t) (void*) wire_join_federation);
    dummy_var ^= ((int64_t) (void*) new_uint_8_list_0);
    dummy_var ^= ((int64_t) (void*) free_WireSyncReturn);
    dummy_var ^= ((int64_t) (void*) store_dart_post_cobject);
    dummy_var ^= ((int64_t) (void*) get_dart_object);
    dummy_var ^= ((int64_t) (void*) drop_dart_object);
    dummy_var ^= ((int64_t) (void*) new_dart_opaque);
    return dummy_var;
}
