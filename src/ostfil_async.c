#include "../include/ostfil.h"
#include <string.h>
#include <stdlib.h>
#include <uv.h>

typedef struct {
    char file_dist[1024];
    char file_path[1024];
    char file_name[64];
    int slice_sum;
    ostfil_cb_t cb;
    void *user_data;
} write_req_t;

// 异步写入回调
static void on_open_write(uv_fs_t *req) {
    write_req_t *wreq = (write_req_t *)req->data;
    uv_fs_req_cleanup(req);
    free(req);
    free(wreq);
}

// 异步读取回调
static void on_open_read(uv_fs_t *req) {
    write_req_t *rreq = (write_req_t *)req->data;
    uv_fs_req_cleanup(req);
    free(req);
    free(rreq);
}

// 异步删除回调
static void on_unlink(uv_fs_t *req) {
    write_req_t *dreq = (write_req_t *)req->data;
    uv_fs_req_cleanup(req);
    free(req);
    free(dreq);
}

// 异步写入
void ostfil_write_async(const char *file_dist,
                        const char *file_path,
                        char *file_name,
                        int *slice_sum,
                        ostfil_cb_t cb,
                        void *user_data)
{
    write_req_t *wreq = malloc(sizeof(write_req_t));
    strcpy(wreq->file_dist, file_dist);
    strcpy(wreq->file_path, file_path);
    wreq->cb = cb;
    wreq->user_data = user_data;
    wreq->slice_sum = 0;

    // 直接调用 uuid4_generate
    uuid4_generate(wreq->file_name);

    // 异步打开源文件
    open_async(file_dist, O_RDONLY, 0, on_open_write, wreq);
}

// 异步读取
void ostfil_read_async(const char *file_path,
                       const char *file_name,
                       const char *file_dist,
                       ostfil_cb_t cb,
                       void *user_data)
{
    write_req_t *rreq = malloc(sizeof(write_req_t));
    strcpy(rreq->file_path, file_path);
    strcpy(rreq->file_name, file_name);
    strcpy(rreq->file_dist, file_dist);
    rreq->cb = cb;
    rreq->user_data = user_data;

    open_async(file_path, O_RDONLY, 0, on_open_read, rreq);
}

// 异步删除
void ostfil_del_async(const char *file_path,
                      const char *file_name,
                      ostfil_cb_t cb,
                      void *user_data)
{
    write_req_t *dreq = malloc(sizeof(write_req_t));
    strcpy(dreq->file_path, file_path);
    strcpy(dreq->file_name, file_name);
    dreq->cb = cb;
    dreq->user_data = user_data;

    char full_path[1024];
    snprintf(full_path, sizeof(full_path), "%s/%s", file_path, file_name);
    uv_fs_t *req = malloc(sizeof(uv_fs_t));
    req->data = dreq;
    uv_fs_unlink(uv_default_loop(), req, full_path, on_unlink);
}
