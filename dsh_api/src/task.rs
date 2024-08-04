use crate::DshApiClient;

// /allocation/{tenant}/task                                                get
// /allocation/{tenant}/task/{appid}                                        get
// /allocation/{tenant}/task/{appid}/{id}                                   get
// /allocation/{tenant}/task/{appid}/{id}/actual                            get
// /allocation/{tenant}/task/{appid}/{id}/status                            get

impl DshApiClient<'_> {
  // pub async fn _get_tasks(&self, service_name: &str) -> DshApiResult<HashMap<TaskId, TaskStatus>> {
  //   // TODO Panics
  //   match self.get_all_derived_task_ids(service_name).await {
  //     Ok(task_ids) => Ok(
  //       join_all(task_ids.iter().map(|task_id| async {
  //         let task_id = TaskId::try_from(task_id.clone()).unwrap();
  //         (task_id.clone(), self.get_derived_task_status(service_name, &task_id).await.unwrap())
  //       }))
  //       .await
  //       .into_iter()
  //       .collect::<HashMap<TaskId, TaskStatus>>(),
  //     ),
  //     Err(_) => Err("".to_string()),
  //   }
  // }
}
