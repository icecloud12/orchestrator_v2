use std::fmt::Display;

use tokio_postgres::{types::Type, Error, Row};

use crate::utils::postgres_utils::POSTGRES_CLIENT;

use super::{
    container_instance_port_pool_junction::ContainerInstancePortPoolJunctionColumns,
    containers::ServiceContainerColumns,
    load_balancer_container_junctions::LoadBalancerContainerJunctionColumns,
    port_pool::PortPoolColumns, tables::TABLES,
};

pub enum ServiceLoadBalancersColumns {
    ID,
    IMAGE_FK,
    HEAD,
    BEHAVIOR,
}
impl Display for ServiceLoadBalancersColumns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ID => write!(f, "id"),
            Self::IMAGE_FK => write!(f, "image_fk"),
            Self::HEAD => write!(f, "head"),
            Self::BEHAVIOR => write!(f, "behavior"),
        }
    }
}

impl ServiceLoadBalancersColumns {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::ID => "id",
            Self::IMAGE_FK => "image_fk",
            Self::HEAD => "head",
            Self::BEHAVIOR => "behavior",
        }
    }
}

pub async fn get_existing_load_balancer_by_image(image_fk: &i32) -> Result<Vec<Row>, Error> {
    let client = POSTGRES_CLIENT.get().unwrap();
    client
        .query_typed(
            format!(
                "
            SELECT
                lb.{lb_id} as lb_id,    
                lb.{lb_head} as lb_head,
                c.{c_id} as c_id,
                c.{c_docker_container_id},
                c.{c_cippj_fk}
                pp.{pp_port},
                cippj.{cippj_uuid},
            FROM
                {lb_table} lb
            LEFT JOIN
                {lbcj_table} lbcj ON lb.{lb_id} = lbcj.{lbcj_lbfk}
            LEFT JOIN
                {c_table} c ON lbcj.{lbcj_cfk} = c.{c_id}
            LEFT JOIN
                {cippj_table} cippj ON c.{c_cippjfk} = cippj.{cippj_id}
            LEFT JOIN
                {pp_table} pp ON cippj.{cippj_ppfk} = pp.{pp_id}
            WHERE
                lb.{lb_image_fk} = $1 AND
                cippj.{cippj_inuse} = true
        ",
                lb_id = ServiceLoadBalancersColumns::ID,
                lb_head = ServiceLoadBalancersColumns::HEAD,
                c_id = ServiceContainerColumns::ID,
                c_docker_container_id = ServiceContainerColumns::DOCKER_CONTAINER_ID,
                c_cippj_fk = ServiceContainerColumns::CONTAINER_INSTANCE_PORT_POOL_JUNCTION_FK,
                pp_port = PortPoolColumns::PORT,
                cippj_uuid = ContainerInstancePortPoolJunctionColumns::UUID,
                lb_table = TABLES::SERVICE_LOADBALANCERS,
                lbcj_table = TABLES::LOAD_BALANCER_CONTAINER_JUNCTION,
                lbcj_lbfk = LoadBalancerContainerJunctionColumns::LOAD_BALANCER_FK,
                c_table = TABLES::SERVICE_CONTAINER,
                lbcj_cfk = LoadBalancerContainerJunctionColumns::CONTAINER_FK,
                cippj_table = TABLES::CONTAINER_INSTANCE_PORT_POOL_JUNCTION,
                c_cippjfk = ServiceContainerColumns::CONTAINER_INSTANCE_PORT_POOL_JUNCTION_FK,
                cippj_id = ContainerInstancePortPoolJunctionColumns::ID,
                pp_table = TABLES::PORT_POOL,
                cippj_ppfk = ContainerInstancePortPoolJunctionColumns::PORT_POOL_FK,
                pp_id = PortPoolColumns::ID,
                lb_image_fk = ServiceLoadBalancersColumns::IMAGE_FK,
                cippj_inuse = ContainerInstancePortPoolJunctionColumns::IN_USE,
            )
            .as_str(),
            &[(image_fk, Type::INT4)],
        )
        .await
}
