use axum::Router;

/// Trait representing a type that can be turned into
/// an [`Router`].
///
/// <table>
///     <tr>
///         <td style="background-color:green;color:black;">
///         Requires the <span style="color:blue">axum</span>
///         feature flag.
///         </td>
///     </tr>
/// </table>
pub trait IntoAxumRouter {
    /// Converts the store into an [`Router`]
    /// that will handle all standard REST request to realize CRUD operations
    /// on the store:
    ///
    /// - `GET /` will list all entities,
    ///   - expected request payload: none
    ///   - returned response body: `Json<Vec<WithId<E,i64>>>`
    /// - `POST /` will create a new entity,
    ///   - expected request payload: `Json<E>`
    ///   - returned response body: `Json<WithId<E,i64>>`
    /// - `PUT /` will update an existing entity,
    ///   - expected request payload: `Json<WithId<E,i64>>`
    ///   - returned response body: `Json<WithId<E,i64>>`
    /// - `DELETE /` will delete all entities
    ///   - expected request payload: none
    ///   - returned response body: none
    /// - `GET /:id` to retrieve one entity from the store
    ///   - expected request payload: none
    ///   - returned response body: `Json<E>`
    /// - `PUT /:id` to update one entity in the store
    ///   - expected request payload: `Json<E>`
    ///   - returned response body: `Json<WithId<E,i64>>`
    /// - `DELETE /:id` to delete one entity from the store
    ///   - expected request payload: none
    ///   - returned response body: none
    fn into_axum_router<S>(self) -> Router<S>;
}
