#![cfg(feature = "redis")]

use tokio::time::{sleep, Duration};

use nimble_web::redis::redis_lock::{DistributedLock, InMemoryLock, LockError};

#[tokio::test]
async fn in_memory_lock_acquire_and_release() {
    let lock = InMemoryLock::new();
    let token = lock.try_lock("key", 10).await.unwrap().expect("token");
    lock.unlock("key", &token).await.unwrap();
    let reacquired = lock.try_lock("key", 5).await.unwrap();
    assert!(reacquired.is_some());
}

#[tokio::test]
async fn in_memory_lock_conflict_when_locked() {
    let lock = InMemoryLock::new();
    let token = lock.try_lock("key", 10).await.unwrap().expect("token");
    let conflict = lock.try_lock("key", 5).await.unwrap();
    assert!(conflict.is_none());
    lock.unlock("key", &token).await.unwrap();
}

#[tokio::test]
async fn in_memory_lock_unlock_fails_on_bad_token() {
    let lock = InMemoryLock::new();
    let token = lock.try_lock("key", 10).await.unwrap().expect("token");
    let wrong = "bad";
    let error = lock.unlock("key", wrong).await;
    assert!(matches!(error, Err(LockError::Conflict(_))));
    lock.unlock("key", &token).await.unwrap();
}

#[tokio::test]
async fn in_memory_lock_ttl_expires() {
    let lock = InMemoryLock::new();
    let _token = lock.try_lock("key", 1).await.unwrap().expect("token");
    sleep(Duration::from_secs(2)).await;
    let reacquire = lock.try_lock("key", 5).await.unwrap();
    assert!(reacquire.is_some());
    lock.unlock("key", &reacquire.unwrap()).await.unwrap();
}
