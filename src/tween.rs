//! Module containg implementations for tween
//!
//! This crate currently have 2 tween implementation which is:
//! - [`Tween`], containg information about a target and an interpolator which
//!   uses generic all the way.
//!   Built-in supported [`TweenTarget`]s are:
//!   - [`TargetComponent`]
//!   - [`TargetResource`]
//!   - [`TargetAsset`]
//!   
//! - [`TweenDyn`], like [`Tween`] but the inner [`Interpolator`] is boxed and
//!   uses dynamic dispatch which you can use closure as an [`Interpolator`]
//!   and reduce system registration!.
//!
//! See available interpolator in [`interpolate`].
//!
//! [`interpolate`]: crate::interpolate

use bevy::prelude::*;
use std::{marker::PhantomData, time::Duration};

use crate::interpolate::Interpolator;
use crate::tween_timer::AnimationDirection;
use std::any::type_name;

/// [`TweenState`] should be automatically managed by a tween player.
/// User just have to add this component to a tween entity and an assigned
/// tween player will take care of it.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component, Reflect,
)]
#[reflect(Component)]
pub struct TweenState {
    /// `local_elasped` is None meaning that the tween playing that is managing this
    /// tween hasn't/has elasped pass this tween.
    pub local_elasped: Option<Duration>,
    #[allow(missing_docs)]
    pub local_previous_elasped: Option<Duration>,
    /// Direction of currently elasped time
    pub direction: AnimationDirection,
    /// Maximum duration of the this tween.
    pub local_end: Duration,
}

/// Automatically managed by an [`Interpolation`] such as [`EaseFunction`] and
/// [`EaseClosure`] when a tween has the component `TweenState`.
/// See [`sample_interpolations_system`]
///
/// [`sample_interpolations_system`]: crate::interpolation::sample_interpolations_system
/// [`Interpolation`]: crate::interpolation::Interpolation
/// [`EaseFunction`]: crate::interpolation::EaseFunction
/// [`EaseClosure`]: crate::interpolation::EaseClosure
#[derive(Debug, Component, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)] // might want to use sparseset but i'm not sure yet
pub struct TweenInterpolationValue(pub f32);

#[derive(
    Debug, Default, Component, Clone, Copy, PartialEq, Eq, Hash, Reflect,
)]
#[reflect(Component)]
pub struct Tween<T, I>
where
    T: TweenTarget,
    I: Interpolator<Item = T::Item>,
{
    #[allow(missing_docs)]
    pub target: T,
    #[allow(missing_docs)]
    pub interpolator: I,
}
impl<T, I> Tween<T, I>
where
    T: TweenTarget,
    I: Interpolator<Item = T::Item>,
{
    /// Create a new [`Tween`] with a target and an interpolator.
    pub fn new_target<G>(target: G, interpolator: I) -> Self
    where
        G: Into<T>,
    {
        Tween {
            interpolator,
            target: target.into(),
        }
    }
}

impl<T, I> Tween<T, I>
where
    T: TweenTarget + Default,
    I: Interpolator<Item = T::Item>,
{
    /// Create a new [`Tween`] with the default target and an interpolator.
    pub fn new(interpolator: I) -> Self {
        Tween::new_target(T::default(), interpolator)
    }
}

type InterpolatorDyn<Item> = Box<dyn Interpolator<Item = Item>>;
// type InterpolatorReflectedDyn<I> = Box<dyn InterpolatorReflected<Item = I>>;

/// Useful for the implementor to specify what this *target* will return the
/// tweenable [`Self::Item`] which should match [`Interpolator::Item`].
/// See [`TargetComponent`], [`TargetResource`], and [`TargetAsset`].
pub trait TweenTarget {
    /// Type to be interpolated
    type Item;
}

/// Convenient alias for [`Tween`] that [`TargetComponent`] with generic [`Interpolator`].
pub type ComponentTween<I> =
    Tween<TargetComponent<<I as Interpolator>::Item>, I>;

/// Convenient alias for [`Tween`] that [`TargetComponent`] with dyanmic [`Interpolator`].
pub type ComponentTweenDyn<C> = Tween<TargetComponent<C>, InterpolatorDyn<C>>;

/// Tell the tween what component of what entity to tween.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum TargetComponent<C>
where
    C: Component,
{
    /// Target the entity that contains this tween's tween player.
    TweenPlayerEntity(#[reflect(ignore)] PhantomData<C>),
    /// Target the parent of this tween's tween player.
    TweenPlayerParent(#[reflect(ignore)] PhantomData<C>),
    /// Target this entity.
    Entity(Entity, #[reflect(ignore)] PhantomData<C>),
    /// Target these entities.
    Entities(Vec<Entity>, #[reflect(ignore)] PhantomData<C>),
}

impl<C> TargetComponent<C>
where
    C: Component,
{
    /// Target the entity that contains this tween's tween player.
    pub fn tween_player_entity() -> TargetComponent<C> {
        TargetComponent::TweenPlayerEntity(PhantomData)
    }
    /// Target the parent of this tween's tween_player.
    pub fn tween_player_parent() -> TargetComponent<C> {
        TargetComponent::TweenPlayerParent(PhantomData)
    }
    /// Target this entity.
    pub fn entity(entity: Entity) -> TargetComponent<C> {
        TargetComponent::Entity(entity, PhantomData)
    }
    /// Target these entities.
    pub fn entities<I>(entities: I) -> TargetComponent<C>
    where
        I: IntoIterator<Item = Entity>,
    {
        TargetComponent::from_iter(entities)
    }
}

impl<C> TweenTarget for TargetComponent<C>
where
    C: Component,
{
    type Item = C;
}

impl<C> Default for TargetComponent<C>
where
    C: Component,
{
    fn default() -> Self {
        TargetComponent::tween_player_entity()
    }
}

impl<C> From<Entity> for TargetComponent<C>
where
    C: Component,
{
    fn from(value: Entity) -> Self {
        TargetComponent::entity(value)
    }
}

impl<C> FromIterator<Entity> for TargetComponent<C>
where
    C: Component,
{
    fn from_iter<T: IntoIterator<Item = Entity>>(iter: T) -> Self {
        TargetComponent::Entities(iter.into_iter().collect(), PhantomData)
    }
}

impl<C, const N: usize> From<[Entity; N]> for TargetComponent<C>
where
    C: Component,
{
    fn from(value: [Entity; N]) -> Self {
        TargetComponent::entities(value)
    }
}

impl<C> From<Vec<Entity>> for TargetComponent<C>
where
    C: Component,
{
    fn from(value: Vec<Entity>) -> Self {
        TargetComponent::entities(value)
    }
}

impl<C> From<&Vec<Entity>> for TargetComponent<C>
where
    C: Component,
{
    fn from(value: &Vec<Entity>) -> Self {
        TargetComponent::entities(value.iter().copied())
    }
}

impl<C> From<&[Entity]> for TargetComponent<C>
where
    C: Component,
{
    fn from(value: &[Entity]) -> Self {
        TargetComponent::entities(value.iter().copied())
    }
}

impl<C, const N: usize> From<&[Entity; N]> for TargetComponent<C>
where
    C: Component,
{
    fn from(value: &[Entity; N]) -> Self {
        TargetComponent::entities(value.iter().copied())
    }
}

/// A tween player must have this marker within the entity to let
/// [`ComponentTween`]s' system correctly search for the player that owns them.
#[derive(Debug, Default, PartialEq, Eq, Hash, Component, Reflect)]
pub struct TweenPlayerMarker;

impl<I> ComponentTween<I>
where
    I: Interpolator,
    I::Item: Component,
{
    /// Convenient method for targetting tween player's entity.
    pub fn player_entity(interpolator: I) -> Self {
        ComponentTween::new_target(
            TargetComponent::tween_player_entity(),
            interpolator,
        )
    }

    /// Convenient method for targetting tween player's parent.
    pub fn player_parent(interpolator: I) -> Self {
        ComponentTween::new_target(
            TargetComponent::tween_player_parent(),
            interpolator,
        )
    }
}

// /// Tween any [`Tween`] with any [`Interpolator`] that [`TargetComponent`] with
// /// value provided by [`TweenInterpolationValue`] component.
pub fn component_tween_system_full<C, I>(
    q_tween_player: Query<(Option<&Parent>, Has<TweenPlayerMarker>)>,
    q_tween: Query<(
        Entity,
        &Tween<TargetComponent<C>, I>,
        &TweenInterpolationValue,
    )>,
    mut q_component: Query<&mut I::Item>,
) where
    C: Component,
    I: Interpolator<Item = C> + Send + Sync + 'static,
{
    q_tween.iter().for_each(|(entity, tween, ease_value)| {
        let target = match &tween.target {
            TargetComponent::TweenPlayerEntity(_) => {
                match q_tween_player.get(entity) {
                    Ok((_, true)) => entity,
                    Ok((Some(this_parent), false)) => {
                        match q_tween_player.get(this_parent.get()) {
                            Ok((_, true)) => this_parent.get(),
                            _ => return,
                        }
                    }
                    _ => return,
                }
            }
            TargetComponent::TweenPlayerParent(_) => {
                match q_tween_player.get(entity) {
                    Ok((Some(this_parent), true)) => this_parent.get(),
                    Ok((Some(this_parent), false)) => {
                        match q_tween_player.get(this_parent.get()) {
                            Ok((Some(player_parent), true)) => {
                                player_parent.get()
                            }
                            _ => return,
                        }
                    }
                    _ => return,
                }
            }
            TargetComponent::Entity(e, _) => *e,
            TargetComponent::Entities(e, _) => {
                for &target in e {
                    let mut target_component = match q_component.get_mut(target)
                    {
                        Ok(target_component) => target_component,
                        Err(e) => {
                            warn!(
                                "{} query error: {e}",
                                type_name::<ComponentTween<I>>()
                            );
                            continue;
                        }
                    };
                    tween
                        .interpolator
                        .interpolate(&mut target_component, ease_value.0);
                }
                return;
            }
        };

        let mut target_component = match q_component.get_mut(target) {
            Ok(target_component) => target_component,
            Err(e) => {
                warn!("{} query error: {e}", type_name::<ComponentTween<I>>());
                return;
            }
        };
        tween
            .interpolator
            .interpolate(&mut target_component, ease_value.0);
    })
}

#[rustfmt::skip]
type ComponentTweenSystem<I> = for<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, 'j> fn(bevy::prelude::Query<'a, 'b, (std::option::Option<&'c bevy::prelude::Parent>, bevy::prelude::Has<TweenPlayerMarker>)>, bevy::prelude::Query<'d, 'e, (bevy::prelude::Entity, &'f Tween<TargetComponent<<I as Interpolator>::Item>, I>, &'g TweenInterpolationValue)>, bevy::prelude::Query<'h, 'i, &'j mut <I as Interpolator>::Item>);

pub fn component_tween_system<I>() -> ComponentTweenSystem<I>
where
    I: Interpolator,
    I::Item: Component,
{
    component_tween_system_full::<I::Item, I>
}

#[rustfmt::skip]
type ComponentTweenDynSysytem<C> = for<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, 'j> fn(bevy::prelude::Query<'a, 'b, (std::option::Option<&'c bevy::prelude::Parent>, bevy::prelude::Has<TweenPlayerMarker>)>, bevy::prelude::Query<'d, 'e, (bevy::prelude::Entity, &'f Tween<TargetComponent<C>, std::boxed::Box<(dyn Interpolator<Item = C> + 'static)>>, &'g TweenInterpolationValue)>, bevy::prelude::Query<'h, 'i, &'j mut <std::boxed::Box<(dyn Interpolator<Item = C> + 'static)> as Interpolator>::Item>);

pub fn component_tween_dyn_system<C>() -> ComponentTweenDynSysytem<C>
where
    C: Component,
{
    component_tween_system_full::<C, InterpolatorDyn<C>>
}

/// Convenient alias for [`Tween`] that [`TargetResource`] with generic [`Interpolator`].
pub type ResourceTween<I> = Tween<TargetResource<<I as Interpolator>::Item>, I>;

/// Convenient alias for [`Tween`] that [`TargetResource`] with dyanmic [`Interpolator`].
pub type ResourceTweenDyn<R> = Tween<TargetResource<R>, InterpolatorDyn<R>>;

/// Tell the tween what resource to tween.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct TargetResource<R>(#[reflect(ignore)] pub PhantomData<R>)
where
    R: Resource;

impl<R> TargetResource<R>
where
    R: Resource,
{
    /// New resource target
    pub fn new() -> TargetResource<R> {
        TargetResource(PhantomData)
    }
}

impl<R> TweenTarget for TargetResource<R>
where
    R: Resource,
{
    type Item = R;
}

// /// Tween any [`ResourceTween`] with value provided by [`TweenInterpolationValue`] component.
pub fn resource_tween_system_full<R, I>(
    q_tween: Query<(&Tween<TargetResource<R>, I>, &TweenInterpolationValue)>,
    resource: Option<ResMut<I::Item>>,
) where
    R: Resource,
    I: Interpolator<Item = R> + Send + Sync + 'static,
{
    let Some(mut resource) = resource else {
        warn!("Resource does not exists for a resource tween.");
        return;
    };
    q_tween.iter().for_each(|(tween, ease_value)| {
        tween.interpolator.interpolate(&mut resource, ease_value.0);
    })
}

#[rustfmt::skip]
type ResourceTweenSystem<I> = for<'a, 'b, 'c, 'd, 'e> fn(bevy::prelude::Query<'a, 'b, (&'c Tween<TargetResource<<I as Interpolator>::Item>, I>, &'d TweenInterpolationValue)>, std::option::Option<bevy::prelude::ResMut<'e, <I as Interpolator>::Item>>);
pub fn resource_tween_system<I>() -> ResourceTweenSystem<I>
where
    I: Interpolator,
    I::Item: Resource,
{
    resource_tween_system_full::<I::Item, I>
}

#[rustfmt::skip]
type ResourceTweenDynSysytem<R> = for<'a, 'b, 'c, 'd, 'e> fn(bevy::prelude::Query<'a, 'b, (&'c Tween<TargetResource<R>, std::boxed::Box<(dyn Interpolator<Item = R> + 'static)>>, &'d TweenInterpolationValue)>, std::option::Option<bevy::prelude::ResMut<'e, <std::boxed::Box<(dyn Interpolator<Item = R> + 'static)> as Interpolator>::Item>>);
pub fn resource_tween_dyn_system<R>() -> ResourceTweenDynSysytem<R>
where
    R: Resource,
{
    resource_tween_system_full::<R, InterpolatorDyn<R>>
}

/// Convenient alias for [`Tween`] that [`TargetAsset`] with generic [`Interpolator`].
#[cfg(feature = "bevy_asset")]
pub type AssetTween<I> = Tween<TargetAsset<<I as Interpolator>::Item>, I>;

/// Convenient alias for [`Tween`] that [`TargetAsset`] with dyanmic [`Interpolator`].
#[cfg(feature = "bevy_asset")]
pub type AssetTweenDyn<A> = Tween<TargetAsset<A>, InterpolatorDyn<A>>;

/// Tell the tween what asset of what type to tween.
#[cfg(feature = "bevy_asset")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum TargetAsset<A: Asset>
where
    A: Asset,
{
    /// Target this asset
    Asset(Handle<A>),
    /// Target these assets
    Assets(Vec<Handle<A>>),
}

#[cfg(feature = "bevy_asset")]
impl<A: Asset> TargetAsset<A> {
    /// Target this asset
    pub fn asset(asset: Handle<A>) -> Self {
        TargetAsset::Asset(asset)
    }

    /// Target these assets
    pub fn assets<I>(assets: I) -> Self
    where
        I: IntoIterator<Item = Handle<A>>,
    {
        TargetAsset::from_iter(assets)
    }
}

#[cfg(feature = "bevy_asset")]
impl<A: Asset> TweenTarget for TargetAsset<A> {
    type Item = A;
}

#[cfg(feature = "bevy_asset")]
impl<A: Asset> Default for TargetAsset<A> {
    fn default() -> Self {
        TargetAsset::Asset(Default::default())
    }
}

#[cfg(feature = "bevy_asset")]
impl<A: Asset> From<Handle<A>> for TargetAsset<A> {
    fn from(value: Handle<A>) -> Self {
        TargetAsset::Asset(value)
    }
}

#[cfg(feature = "bevy_asset")]
impl<A: Asset> FromIterator<Handle<A>> for TargetAsset<A> {
    fn from_iter<T: IntoIterator<Item = Handle<A>>>(iter: T) -> Self {
        TargetAsset::Assets(iter.into_iter().collect())
    }
}

#[cfg(feature = "bevy_asset")]
impl<A: Asset, const N: usize> From<[Handle<A>; N]> for TargetAsset<A> {
    fn from(value: [Handle<A>; N]) -> Self {
        TargetAsset::assets(value)
    }
}

#[cfg(feature = "bevy_asset")]
impl<A: Asset> From<Vec<Handle<A>>> for TargetAsset<A> {
    fn from(value: Vec<Handle<A>>) -> Self {
        TargetAsset::assets(value)
    }
}

#[cfg(feature = "bevy_asset")]
impl<A: Asset> From<&Vec<Handle<A>>> for TargetAsset<A> {
    fn from(value: &Vec<Handle<A>>) -> Self {
        TargetAsset::assets(value.iter().cloned())
    }
}

#[cfg(feature = "bevy_asset")]
impl<A: Asset> From<&[Handle<A>]> for TargetAsset<A> {
    fn from(value: &[Handle<A>]) -> Self {
        TargetAsset::assets(value.iter().cloned())
    }
}

#[cfg(feature = "bevy_asset")]
impl<A: Asset, const N: usize> From<&[Handle<A>; N]> for TargetAsset<A> {
    fn from(value: &[Handle<A>; N]) -> Self {
        TargetAsset::assets(value.iter().cloned())
    }
}

/// Tween any [`AssetTween`] with value provided by [`TweenInterpolationValue`] component.
#[cfg(feature = "bevy_asset")]
pub fn asset_tween_system_full<A, I>(
    q_tween: Query<(&Tween<TargetAsset<A>, I>, &TweenInterpolationValue)>,
    asset: Option<ResMut<Assets<I::Item>>>,
) where
    A: Asset,
    I: Interpolator<Item = A> + Send + Sync + 'static,
{
    let Some(mut asset) = asset else {
        warn!("Asset resource does not exists for an asset tween.");
        return;
    };
    q_tween
        .iter()
        .for_each(|(tween, ease_value)| match &tween.target {
            TargetAsset::Asset(a) => {
                let Some(asset) = asset.get_mut(a) else {
                    warn!("Asset not found for an asset tween");
                    return;
                };
                tween.interpolator.interpolate(asset, ease_value.0);
            }
            TargetAsset::Assets(assets) => {
                for a in assets {
                    let Some(a) = asset.get_mut(a) else {
                        warn!("Asset not found for an asset tween");
                        continue;
                    };
                    tween.interpolator.interpolate(a, ease_value.0);
                }
            }
        })
}

#[cfg(feature = "bevy_asset")]
#[rustfmt::skip]
type AssetTweenSystem<I> = for<'a, 'b, 'c, 'd, 'e> fn(bevy::prelude::Query<'a, 'b, (&'c Tween<TargetAsset<<I as Interpolator>::Item>, I>, &'d TweenInterpolationValue)>, std::option::Option<bevy::prelude::ResMut<'e, bevy::prelude::Assets<<I as Interpolator>::Item>>>);
#[cfg(feature = "bevy_asset")]
pub fn asset_tween_system<I>() -> AssetTweenSystem<I>
where
    I: Interpolator,
    I::Item: Asset,
{
    asset_tween_system_full::<I::Item, I>
}

#[cfg(feature = "bevy_asset")]
#[rustfmt::skip]
type AssetTweenDynSysytem<A> = for<'a, 'b, 'c, 'd, 'e> fn(bevy::prelude::Query<'a, 'b, (&'c Tween<TargetAsset<A>, std::boxed::Box<(dyn Interpolator<Item = A> + 'static)>>, &'d TweenInterpolationValue)>, std::option::Option<bevy::prelude::ResMut<'e, bevy::prelude::Assets<<std::boxed::Box<(dyn Interpolator<Item = A> + 'static)> as Interpolator>::Item>>>);
#[cfg(feature = "bevy_asset")]
pub fn asset_tween_dyn_system<A>() -> AssetTweenDynSysytem<A>
where
    A: Asset,
{
    asset_tween_system_full::<A, InterpolatorDyn<A>>
}
